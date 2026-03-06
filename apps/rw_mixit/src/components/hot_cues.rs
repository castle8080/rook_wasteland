/// Hot cues component: 4 instant-jump buttons per deck.
///
/// # Task coverage
/// - T7.1 — 4 colored buttons (red, blue, green, yellow); dim = unset, lit = set.
/// - T7.2 — Hold ≥ 300 ms → sets `hot_cues[index] = Some(current_secs)`.
/// - T7.3 — Tap (< 300 ms) on a set cue → seeks to saved position.
/// - T7.4 — Double-tap (two taps ≤ 400 ms apart) or right-click → clears cue.
use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::audio::deck_audio::AudioDeck;
use crate::state::DeckState;

/// Shorthand for the `Closure` cell shared between `pointerdown` and the hold timer.
type HoldClosureCell = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

/// Accent colors for hot cue buttons 0–3.
const HC_COLORS: [&str; 4] = ["#ef4444", "#3b82f6", "#22c55e", "#eab308"];
/// Display labels for hot cue buttons.
const HC_LABELS: [&str; 4] = ["1", "2", "3", "4"];

/// Returns the current `performance.now()` timestamp in milliseconds, or 0.0
/// on failure.
fn perf_now() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now())
        .unwrap_or(0.0)
}

/// Cancel a pending `setTimeout` identified by the stored handle (if any).
fn clear_timeout(handle: &Rc<RefCell<Option<i32>>>) {
    if let Some(h) = handle.borrow_mut().take() {
        if let Some(w) = web_sys::window() {
            w.clear_timeout_with_handle(h);
        }
    }
}

/// Hot cue buttons for one deck.
///
/// `audio_deck_holder` is required so the tap-to-jump handler can call `seek`.
#[component]
pub fn HotCues(
    state:             DeckState,
    audio_deck_holder: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
) -> impl IntoView {
    let buttons = (0usize..4).map(|idx| {
        // ── Per-button interaction state ──────────────────────────────────────
        // setTimeout handle — cleared after the timer fires or is cancelled.
        let hold_handle:  Rc<RefCell<Option<i32>>> = Rc::new(RefCell::new(None));
        // The Closure kept alive while the hold timer is pending.
        let hold_closure: HoldClosureCell          = Rc::new(RefCell::new(None));
        // True when the hold timer fired before pointerup cancelled it.
        let hold_fired:   Rc<RefCell<bool>>                         = Rc::new(RefCell::new(false));
        // perf_now() at the last pointerdown, used to detect taps vs holds.
        let down_at:      Rc<RefCell<f64>>                          = Rc::new(RefCell::new(0.0));
        // perf_now() of the most recent successful tap (for double-tap detection).
        let last_tap_at:  Rc<RefCell<f64>>                          = Rc::new(RefCell::new(0.0));

        // ── pointerdown — start hold timer ────────────────────────────────────
        let on_down = {
            let state        = state.clone();
            let hold_handle  = hold_handle.clone();
            let hold_closure = hold_closure.clone();
            let hold_fired   = hold_fired.clone();
            let down_at      = down_at.clone();
            move |ev: web_sys::PointerEvent| {
                // Ignore non-primary button (right-click is handled by contextmenu).
                if ev.button() != 0 { return; }

                // Cancel any timer left over from a previous pointerdown that
                // fired without a matching pointerup/leave (e.g. pointer capture
                // loss). Without this, the stale timer would fire and overwrite
                // the cue with an old playhead position.
                clear_timeout(&hold_handle);
                *hold_closure.borrow_mut() = None;

                *down_at.borrow_mut()    = perf_now();
                *hold_fired.borrow_mut() = false;

                let state_t      = state.clone();
                let hold_fired_t = hold_fired.clone();
                let hold_handle_t = hold_handle.clone();

                let timer_cb = Closure::<dyn FnMut()>::new(move || {
                    *hold_fired_t.borrow_mut()  = true;
                    *hold_handle_t.borrow_mut() = None;
                    let pos = state_t.current_secs.get_untracked();
                    state_t.hot_cues.update(|cues| cues[idx] = Some(pos));
                    // NOTE: hold_closure is NOT cleared here — the pointerup/leave
                    // handler drops it after the timer fires.
                });

                let Some(window) = web_sys::window() else { return };
                match window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    timer_cb.as_ref().unchecked_ref(),
                    300,
                ) {
                    Ok(handle) => {
                        *hold_handle.borrow_mut()  = Some(handle);
                        *hold_closure.borrow_mut() = Some(timer_cb);
                    }
                    // setTimeout failure is non-critical; just skip the hold path.
                    Err(_) => timer_cb.forget(),
                }
            }
        };

        // ── pointerup — tap, double-tap, or post-hold cleanup ─────────────────
        let on_up = {
            let state             = state.clone();
            let hold_handle       = hold_handle.clone();
            let hold_closure      = hold_closure.clone();
            let hold_fired        = hold_fired.clone();
            let last_tap_at       = last_tap_at.clone();
            let audio_deck_holder = audio_deck_holder.clone();
            move |ev: web_sys::PointerEvent| {
                if ev.button() != 0 { return; }

                // Cancel pending hold timer (no-op if already fired).
                clear_timeout(&hold_handle);
                *hold_closure.borrow_mut() = None;

                // If the hold timer already fired → cue was set; nothing more to do.
                if *hold_fired.borrow() {
                    *hold_fired.borrow_mut() = false;
                    return;
                }

                // Fast release → tap. Act only when the cue is already set.
                let cue_pos = match state.hot_cues.get_untracked()[idx] {
                    Some(p) => p,
                    None    => return,
                };

                let now  = perf_now();
                let last = *last_tap_at.borrow();
                if last > 0.0 && (now - last) < 400.0 {
                    // Double-tap detected → clear the cue.
                    *last_tap_at.borrow_mut() = 0.0;
                    state.hot_cues.update(|c| c[idx] = None);
                } else {
                    // Single tap → jump to the saved position.
                    *last_tap_at.borrow_mut() = now;
                    if let Some(ref deck_rc) = *audio_deck_holder.borrow() {
                        let rate = state.playback_rate.get_untracked() as f32;
                        deck_rc.borrow_mut().seek(cue_pos, rate);
                        state.current_secs.set(cue_pos);
                    }
                }
            }
        };

        // ── pointerleave — cancel hold timer without triggering tap ───────────
        let on_leave = {
            let hold_handle  = hold_handle.clone();
            let hold_closure = hold_closure.clone();
            let hold_fired   = hold_fired.clone();
            move |_: web_sys::PointerEvent| {
                clear_timeout(&hold_handle);
                *hold_closure.borrow_mut() = None;
                *hold_fired.borrow_mut()   = false;
            }
        };

        // pointercancel mirrors pointerleave.
        let on_cancel = {
            let hold_handle  = hold_handle.clone();
            let hold_closure = hold_closure.clone();
            let hold_fired   = hold_fired.clone();
            move |_: web_sys::PointerEvent| {
                clear_timeout(&hold_handle);
                *hold_closure.borrow_mut() = None;
                *hold_fired.borrow_mut()   = false;
            }
        };

        // ── contextmenu (right-click / long-press) — clear cue ───────────────
        let on_context = {
            let state = state.clone();
            move |ev: web_sys::MouseEvent| {
                ev.prevent_default();
                state.hot_cues.update(|c| c[idx] = None);
            }
        };

        let color       = HC_COLORS[idx];
        let label       = HC_LABELS[idx];
        let hot_cues_s  = state.hot_cues;

        view! {
            <button
                class="btn-hc"
                class:btn-hc-set=move || hot_cues_s.get()[idx].is_some()
                style=move || {
                    let is_set = hot_cues_s.get()[idx].is_some();
                    if is_set {
                        format!("--hc-color:{color}; background-color:{color};")
                    } else {
                        format!("--hc-color:{color};")
                    }
                }
                on:pointerdown=on_down
                on:pointerup=on_up
                on:pointerleave=on_leave
                on:pointercancel=on_cancel
                on:contextmenu=on_context
            >
                {label}
            </button>
        }
    }).collect_view();

    view! {
        <div class="hot-cues">
            {buttons}
        </div>
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Pure logic helpers mirroring the component's interaction rules.

    /// Returns whether a release constitutes a "tap" (fast release vs hold).
    fn is_tap(down_at_ms: f64, up_at_ms: f64) -> bool {
        (up_at_ms - down_at_ms) < 300.0
    }

    /// Returns whether two tap timestamps qualify as a double-tap.
    fn is_double_tap(first_tap_ms: f64, second_tap_ms: f64) -> bool {
        second_tap_ms > first_tap_ms && (second_tap_ms - first_tap_ms) < 400.0
    }

    #[test]
    fn quick_release_is_a_tap() {
        assert!(is_tap(0.0, 150.0));
    }

    #[test]
    fn slow_release_is_not_a_tap() {
        assert!(!is_tap(0.0, 350.0));
    }

    #[test]
    fn exactly_300ms_is_not_a_tap() {
        assert!(!is_tap(0.0, 300.0));
    }

    #[test]
    fn two_taps_within_400ms_is_double_tap() {
        assert!(is_double_tap(100.0, 450.0));
    }

    #[test]
    fn two_taps_beyond_400ms_is_not_double_tap() {
        assert!(!is_double_tap(100.0, 550.0));
    }

    #[test]
    fn exactly_400ms_gap_is_not_double_tap() {
        assert!(!is_double_tap(0.0, 400.0));
    }
}
