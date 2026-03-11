/// Global keyboard shortcut registration for rw_mixit.
///
/// # Key mapping
///
/// | Key            | Action                     |
/// |----------------|----------------------------|
/// | `Space`        | Play / Pause — Deck A      |
/// | `Enter`        | Play / Pause — Deck B      |
/// | `Q`            | Cue — Deck A               |
/// | `P`            | Cue — Deck B               |
/// | `Z`            | Loop In — Deck A           |
/// | `X`            | Loop Out — Deck A          |
/// | `N`            | Loop In — Deck B           |
/// | `M`            | Loop Out — Deck B          |
/// | `ArrowLeft`    | Nudge − — Deck A           |
/// | `ArrowRight`   | Nudge + — Deck A           |
/// | `[`            | Nudge − — Deck B           |
/// | `]`            | Nudge + — Deck B           |
/// | `1`–`4`        | Hot Cues 1–4 — Deck A     |
/// | `7`–`0`        | Hot Cues 1–4 — Deck B     |
///
/// Shortcuts are suppressed when a text input or textarea has focus.
/// Auto-repeated keydown events are ignored for all actions.
use std::rc::Rc;
use std::cell::RefCell;

use gloo_events::EventListener;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

use crate::audio::deck_audio::AudioDeck;
use crate::state::DeckState;

// ── Public API ────────────────────────────────────────────────────────────────

/// Returns `true` when an `INPUT` or `TEXTAREA` element currently has keyboard focus.
///
/// Used to suppress shortcut keys while the user is typing.  Returns `false` on
/// any failure (e.g. unavailable document), so shortcuts remain active rather
/// than being permanently silenced.
pub fn is_input_focused() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element())
        .map(|el| is_tag_input_or_textarea(&el.tag_name()))
        .unwrap_or(false)
}

/// Returns `true` if `tag_name` (case-insensitive) is `"input"` or `"textarea"`.
///
/// Extracted as a pure function so it can be tested natively without a browser.
pub fn is_tag_input_or_textarea(tag: &str) -> bool {
    let upper = tag.to_ascii_uppercase();
    upper == "INPUT" || upper == "TEXTAREA"
}

/// The side of the mixer to which a keyboard shortcut applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckSide { A, B }

/// The action described by a keyboard shortcut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    PlayPause,
    Cue,
    LoopIn,
    LoopOut,
    /// `direction`: `-1` = left/backwards, `+1` = right/forwards.
    NudgeStart(i8),
    /// `index`: 0–3 (maps to hot cue buttons 1–4 on screen).
    HotCue(usize),
}

/// Maps a `KeyboardEvent.code()` string to `(DeckSide, ShortcutAction)`.
///
/// Returns `None` for unrecognised keys.  The result is used both by the
/// runtime listener and by native unit tests that verify the mapping table.
pub fn key_to_action(code: &str) -> Option<(DeckSide, ShortcutAction)> {
    use DeckSide::*;
    use ShortcutAction::*;
    match code {
        // ── Deck A ────────────────────────────────────────────────────────────
        "Space"       => Some((A, PlayPause)),
        "KeyQ"        => Some((A, Cue)),
        "KeyZ"        => Some((A, LoopIn)),
        "KeyX"        => Some((A, LoopOut)),
        "ArrowLeft"   => Some((A, NudgeStart(-1))),
        "ArrowRight"  => Some((A, NudgeStart(1))),
        "Digit1"      => Some((A, HotCue(0))),
        "Digit2"      => Some((A, HotCue(1))),
        "Digit3"      => Some((A, HotCue(2))),
        "Digit4"      => Some((A, HotCue(3))),
        // ── Deck B ────────────────────────────────────────────────────────────
        "Enter"        => Some((B, PlayPause)),
        "KeyP"         => Some((B, Cue)),
        "KeyN"         => Some((B, LoopIn)),
        "KeyM"         => Some((B, LoopOut)),
        "BracketLeft"  => Some((B, NudgeStart(-1))),
        "BracketRight" => Some((B, NudgeStart(1))),
        "Digit7"       => Some((B, HotCue(0))),
        "Digit8"       => Some((B, HotCue(1))),
        "Digit9"       => Some((B, HotCue(2))),
        "Digit0"       => Some((B, HotCue(3))),
        _ => None,
    }
}

/// Whether a shortcut action should call `prevent_default()` to suppress the
/// browser's built-in handling (scroll, form submit, etc.).
fn needs_prevent_default(action: ShortcutAction) -> bool {
    use ShortcutAction::*;
    matches!(action, PlayPause | NudgeStart(_))
}

/// Opaque container that keeps the global `keydown` and `keyup` listeners alive.
///
/// Dropping this value calls `removeEventListener` on both listeners.
/// Call `std::mem::forget` on the returned value to keep shortcuts active for
/// the entire application lifetime.
pub struct KeyboardListeners {
    _keydown: EventListener,
    _keyup:   EventListener,
}

type AudioHolder = Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>;

/// Register the global keyboard shortcut listeners on `window` and return a
/// handle that keeps them alive.
///
/// The caller is responsible for calling `std::mem::forget` on the returned
/// value (or storing it at app-root scope) to prevent the listeners from being
/// removed on the next drop.
pub fn register_keyboard_shortcuts(
    state_a: DeckState,
    audio_a: AudioHolder,
    state_b: DeckState,
    audio_b: AudioHolder,
) -> KeyboardListeners {
    let window = web_sys::window()
        .expect("register_keyboard_shortcuts: window unavailable");

    // Clones for the keyup listener (nudge-end only needs audio holders).
    let audio_a_up = audio_a.clone();
    let audio_b_up = audio_b.clone();

    let keydown = EventListener::new(&window, "keydown", move |ev| {
        if is_input_focused() { return; }
        let ev = ev
            .dyn_ref::<KeyboardEvent>()
            .expect("keydown event is a KeyboardEvent");

        // Suppress auto-repeat; every action should fire at most once per
        // physical key press.
        if ev.repeat() { return; }

        let code = ev.code();
        let Some((side, action)) = key_to_action(&code) else { return };

        if needs_prevent_default(action) {
            ev.prevent_default();
        }

        let (state, holder) = match side {
            DeckSide::A => (&state_a, &audio_a),
            DeckSide::B => (&state_b, &audio_b),
        };

        match action {
            ShortcutAction::PlayPause       => do_play_pause(state, holder),
            ShortcutAction::Cue             => do_cue(state, holder),
            ShortcutAction::LoopIn          => do_loop_in(state),
            ShortcutAction::LoopOut         => do_loop_out(state),
            ShortcutAction::NudgeStart(dir) => do_nudge_start(holder, dir as f32),
            ShortcutAction::HotCue(idx)     => do_hot_cue(state, holder, idx),
        }
    });

    let keyup = EventListener::new(&window, "keyup", move |ev| {
        if is_input_focused() { return; }
        let ev = ev
            .dyn_ref::<KeyboardEvent>()
            .expect("keyup event is a KeyboardEvent");

        let code = ev.code();
        match code.as_str() {
            "ArrowLeft" | "ArrowRight" => do_nudge_end(&audio_a_up),
            "BracketLeft" | "BracketRight" => do_nudge_end(&audio_b_up),
            _ => {}
        }
    });

    KeyboardListeners { _keydown: keydown, _keyup: keyup }
}

// ── Private action helpers ─────────────────────────────────────────────────────

/// Toggle play / pause — mirrors the logic in `Controls::on_play_pause`.
fn do_play_pause(state: &DeckState, holder: &AudioHolder) {
    if let Some(ref deck_rc) = *holder.borrow() {
        let mut deck = deck_rc.borrow_mut();
        if deck.source.is_some() {
            let pos = deck.pause();
            drop(deck);
            state.is_playing.set(false);
            state.current_secs.set(pos);
        } else {
            let offset = deck.offset_at_play;
            let rate   = state.playback_rate.get_untracked() as f32;
            deck.play(offset, rate);
            drop(deck);
            state.is_playing.set(true);
        }
    }
}

/// Seek to the cue point — mirrors `Controls::on_cue`.
fn do_cue(state: &DeckState, holder: &AudioHolder) {
    if let Some(ref deck_rc) = *holder.borrow() {
        let rate = state.playback_rate.get_untracked() as f32;
        deck_rc.borrow_mut().cue(rate);
    }
}

/// Set the loop-in point at the current playhead — mirrors `LoopControls::on_loop_in`.
fn do_loop_in(state: &DeckState) {
    let pos      = state.current_secs.get_untracked();
    let loop_out = state.loop_out.get_untracked();
    state.loop_in.set(pos);
    if state.loop_active.get_untracked() && pos >= loop_out {
        state.loop_active.set(false);
    }
}

/// Set the loop-out point and activate the loop — mirrors `LoopControls::on_loop_out`.
fn do_loop_out(state: &DeckState) {
    let current = state.current_secs.get_untracked();
    let loop_in = state.loop_in.get_untracked();
    // Ensure loop_out is strictly greater than loop_in.
    let out = if current > loop_in { current } else { loop_in + 0.001 };
    state.loop_out.set(out);
    state.loop_active.set(true);
}

/// Begin a pitch nudge — mirrors `Controls` nudge mousedown.
fn do_nudge_start(holder: &AudioHolder, direction: f32) {
    if let Some(ref deck_rc) = *holder.borrow() {
        deck_rc.borrow_mut().nudge_start(direction);
    }
}

/// End the active pitch nudge — mirrors `Controls` nudge mouseup.
fn do_nudge_end(holder: &AudioHolder) {
    if let Some(ref deck_rc) = *holder.borrow() {
        deck_rc.borrow_mut().nudge_end();
    }
}

/// Jump to the hot cue at `idx` if it is set; otherwise set the cue at the
/// current playhead position.
///
/// This diverges from the pointer-based UX (hold = set, tap = jump) in a way
/// that is more natural for keyboard use: a single key press is unambiguous.
fn do_hot_cue(state: &DeckState, holder: &AudioHolder, idx: usize) {
    let cues = state.hot_cues.get_untracked();
    match cues[idx] {
        Some(pos) => {
            if let Some(ref deck_rc) = *holder.borrow() {
                let rate = state.playback_rate.get_untracked() as f32;
                deck_rc.borrow_mut().seek(pos, rate);
                state.current_secs.set(pos);
            }
        }
        None => {
            let pos = state.current_secs.get_untracked();
            state.hot_cues.update(|c| c[idx] = Some(pos));
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_tag_input_or_textarea ──────────────────────────────────────────────

    #[test]
    fn input_tag_is_detected() {
        assert!(is_tag_input_or_textarea("input"));
        assert!(is_tag_input_or_textarea("INPUT"));
        assert!(is_tag_input_or_textarea("Input"));
    }

    #[test]
    fn textarea_tag_is_detected() {
        assert!(is_tag_input_or_textarea("textarea"));
        assert!(is_tag_input_or_textarea("TEXTAREA"));
    }

    #[test]
    fn div_is_not_input_or_textarea() {
        assert!(!is_tag_input_or_textarea("div"));
        assert!(!is_tag_input_or_textarea("BUTTON"));
        assert!(!is_tag_input_or_textarea("SELECT"));
    }

    // ── key_to_action mapping table ───────────────────────────────────────────

    #[test]
    fn space_maps_to_deck_a_play_pause() {
        assert_eq!(key_to_action("Space"), Some((DeckSide::A, ShortcutAction::PlayPause)));
    }

    #[test]
    fn enter_maps_to_deck_b_play_pause() {
        assert_eq!(key_to_action("Enter"), Some((DeckSide::B, ShortcutAction::PlayPause)));
    }

    #[test]
    fn q_maps_to_deck_a_cue() {
        assert_eq!(key_to_action("KeyQ"), Some((DeckSide::A, ShortcutAction::Cue)));
    }

    #[test]
    fn p_maps_to_deck_b_cue() {
        assert_eq!(key_to_action("KeyP"), Some((DeckSide::B, ShortcutAction::Cue)));
    }

    #[test]
    fn z_maps_to_deck_a_loop_in() {
        assert_eq!(key_to_action("KeyZ"), Some((DeckSide::A, ShortcutAction::LoopIn)));
    }

    #[test]
    fn x_maps_to_deck_a_loop_out() {
        assert_eq!(key_to_action("KeyX"), Some((DeckSide::A, ShortcutAction::LoopOut)));
    }

    #[test]
    fn n_maps_to_deck_b_loop_in() {
        assert_eq!(key_to_action("KeyN"), Some((DeckSide::B, ShortcutAction::LoopIn)));
    }

    #[test]
    fn m_maps_to_deck_b_loop_out() {
        assert_eq!(key_to_action("KeyM"), Some((DeckSide::B, ShortcutAction::LoopOut)));
    }

    #[test]
    fn arrow_left_maps_to_deck_a_nudge_minus() {
        assert_eq!(key_to_action("ArrowLeft"), Some((DeckSide::A, ShortcutAction::NudgeStart(-1))));
    }

    #[test]
    fn arrow_right_maps_to_deck_a_nudge_plus() {
        assert_eq!(key_to_action("ArrowRight"), Some((DeckSide::A, ShortcutAction::NudgeStart(1))));
    }

    #[test]
    fn bracket_left_maps_to_deck_b_nudge_minus() {
        assert_eq!(key_to_action("BracketLeft"), Some((DeckSide::B, ShortcutAction::NudgeStart(-1))));
    }

    #[test]
    fn bracket_right_maps_to_deck_b_nudge_plus() {
        assert_eq!(key_to_action("BracketRight"), Some((DeckSide::B, ShortcutAction::NudgeStart(1))));
    }

    #[test]
    fn digit_1_to_4_map_to_deck_a_hot_cues() {
        assert_eq!(key_to_action("Digit1"), Some((DeckSide::A, ShortcutAction::HotCue(0))));
        assert_eq!(key_to_action("Digit2"), Some((DeckSide::A, ShortcutAction::HotCue(1))));
        assert_eq!(key_to_action("Digit3"), Some((DeckSide::A, ShortcutAction::HotCue(2))));
        assert_eq!(key_to_action("Digit4"), Some((DeckSide::A, ShortcutAction::HotCue(3))));
    }

    #[test]
    fn digit_7_to_0_map_to_deck_b_hot_cues() {
        assert_eq!(key_to_action("Digit7"), Some((DeckSide::B, ShortcutAction::HotCue(0))));
        assert_eq!(key_to_action("Digit8"), Some((DeckSide::B, ShortcutAction::HotCue(1))));
        assert_eq!(key_to_action("Digit9"), Some((DeckSide::B, ShortcutAction::HotCue(2))));
        assert_eq!(key_to_action("Digit0"), Some((DeckSide::B, ShortcutAction::HotCue(3))));
    }

    #[test]
    fn unknown_key_returns_none() {
        assert_eq!(key_to_action("KeyA"), None);
        assert_eq!(key_to_action("F5"), None);
        assert_eq!(key_to_action("Tab"), None);
        assert_eq!(key_to_action(""), None);
    }

    #[test]
    fn play_pause_and_nudge_need_prevent_default() {
        assert!(needs_prevent_default(ShortcutAction::PlayPause));
        assert!(needs_prevent_default(ShortcutAction::NudgeStart(1)));
        assert!(needs_prevent_default(ShortcutAction::NudgeStart(-1)));
    }

    #[test]
    fn other_actions_do_not_need_prevent_default() {
        assert!(!needs_prevent_default(ShortcutAction::Cue));
        assert!(!needs_prevent_default(ShortcutAction::LoopIn));
        assert!(!needs_prevent_default(ShortcutAction::LoopOut));
        assert!(!needs_prevent_default(ShortcutAction::HotCue(0)));
    }

    // ── do_loop_in / do_loop_out logic mirrors ────────────────────────────────

    /// Mirrors the deactivation guard in `do_loop_in`.
    fn loop_in_should_deactivate(new_in: f64, loop_out: f64) -> bool {
        new_in >= loop_out
    }

    #[test]
    fn loop_in_at_or_past_loop_out_deactivates() {
        assert!(loop_in_should_deactivate(10.0, 10.0));
        assert!(loop_in_should_deactivate(11.0, 10.0));
    }

    #[test]
    fn loop_in_before_loop_out_does_not_deactivate() {
        assert!(!loop_in_should_deactivate(5.0, 10.0));
    }

    /// Mirrors the clamping in `do_loop_out`.
    fn compute_loop_out(current: f64, loop_in: f64) -> f64 {
        if current > loop_in { current } else { loop_in + 0.001 }
    }

    #[test]
    fn loop_out_past_loop_in_is_unchanged() {
        let out = compute_loop_out(8.0, 5.0);
        assert!((out - 8.0).abs() < 1e-9);
    }

    #[test]
    fn loop_out_at_loop_in_is_nudged_forward() {
        let out = compute_loop_out(5.0, 5.0);
        assert!(out > 5.0);
    }

    #[test]
    fn loop_out_before_loop_in_is_nudged_forward() {
        let out = compute_loop_out(3.0, 5.0);
        assert!(out > 5.0);
    }
}
