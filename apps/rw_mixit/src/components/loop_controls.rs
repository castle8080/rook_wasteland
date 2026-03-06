/// Loop controls component: Loop In, Loop Out, Loop Toggle, and quick bar-length buttons.
///
/// # Task coverage
/// - T6.1 — Loop In button sets `state.loop_in` to the current playhead position.
/// - T6.2 — Loop Out button sets `state.loop_out`; clamps so `out > in`; activates loop.
/// - T6.3 — Loop Toggle button flips `state.loop_active`.
/// - T6.6 — Quick bar-length buttons (½, 1, 2, 4, 8) calculate loop duration from BPM.
///
/// T6.4 (rAF boundary check) and T6.5 (waveform overlay) are handled in
/// `canvas/raf_loop.rs` and `canvas/waveform_draw.rs` respectively.
use leptos::prelude::*;
use crate::state::DeckState;

// ── Pure helpers (exported for tests) ────────────────────────────────────────

/// Compute the duration of a loop region in seconds for the given bar count and BPM.
///
/// Uses 4/4 time: 1 bar = 4 beats = `4 × (60 / bpm)` seconds.
/// Returns 0.0 if `bpm` is not positive.
pub fn bar_loop_duration(bars: f64, bpm: f64) -> f64 {
    if bpm <= 0.0 { return 0.0; }
    bars * 4.0 * (60.0 / bpm)
}

/// Ensure `loop_out` is strictly greater than `loop_in`.
///
/// If `requested_out > loop_in`, returns `requested_out` unchanged.
/// Otherwise returns `loop_in + 0.001` (minimal forward nudge).
pub fn clamp_loop_out(loop_in: f64, requested_out: f64) -> f64 {
    if requested_out > loop_in {
        requested_out
    } else {
        loop_in + 0.001
    }
}

/// Bar lengths available as quick-loop shortcuts (in units of bars).
const BAR_LENGTHS: &[(&str, f64)] = &[
    ("½",  0.5),
    ("1",  1.0),
    ("2",  2.0),
    ("4",  4.0),
    ("8",  8.0),
];

/// Loop controls for one deck.
///
/// `bpm` is the current BPM signal for this deck — required by the quick bar-length
/// buttons. If BPM is None, the bar-length buttons are disabled.
#[component]
pub fn LoopControls(
    state: DeckState,
    /// Detected or tapped BPM for this deck. None = unknown.
    bpm: RwSignal<Option<f64>>,
) -> impl IntoView {
    // ── Loop In (T6.1) ────────────────────────────────────────────────────────
    let on_loop_in = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            let pos      = state.current_secs.get_untracked();
            let loop_out = state.loop_out.get_untracked();
            state.loop_in.set(pos);
            // If the new loop_in is at or beyond the current loop_out the region
            // is invalid.  Deactivate so the UI doesn't show an active-but-silent
            // loop; the user can re-arm by pressing OUT or a bar-length button.
            if state.loop_active.get_untracked() && pos >= loop_out {
                state.loop_active.set(false);
            }
        }
    };

    // ── Loop Out (T6.2) ───────────────────────────────────────────────────────
    let on_loop_out = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            let current  = state.current_secs.get_untracked();
            let loop_in  = state.loop_in.get_untracked();
            state.loop_out.set(clamp_loop_out(loop_in, current));
            state.loop_active.set(true);
        }
    };

    // ── Loop Toggle (T6.3) ────────────────────────────────────────────────────
    let on_loop_toggle = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            let current = state.loop_active.get_untracked();
            state.loop_active.set(!current);
        }
    };

    view! {
        <div class="loop-controls">
            // ── IN / OUT / TOGGLE ─────────────────────────────────────────────
            <div class="loop-transport">
                <button class="btn-loop btn-loop-in" on:click=on_loop_in>
                    "IN"
                </button>
                <button class="btn-loop btn-loop-out" on:click=on_loop_out>
                    "OUT"
                </button>
                <button
                    class="btn-loop btn-loop-toggle"
                    class:loop-active=move || state.loop_active.get()
                    on:click=on_loop_toggle
                >
                    {move || if state.loop_active.get() { "LOOP ●" } else { "LOOP ○" }}
                </button>
            </div>

            // ── Quick bar-length buttons (T6.6) ───────────────────────────────
            <div class="loop-bars">
                <span class="loop-bars-label">"BARS:"</span>
                {BAR_LENGTHS.iter().map(|&(label, bars)| {
                    let state = state.clone();
                    let on_click = move |_: web_sys::MouseEvent| {
                        let Some(bpm_val) = bpm.get_untracked() else { return };
                        if bpm_val <= 0.0 { return }

                        // 1 bar = 4 beats × (60 s / bpm)
                        let loop_length  = bar_loop_duration(bars, bpm_val);
                        let loop_in_pos = state.current_secs.get_untracked();
                        let loop_out_pos = loop_in_pos + loop_length;

                        state.loop_in.set(loop_in_pos);
                        state.loop_out.set(loop_out_pos);
                        state.loop_active.set(true);
                    };

                    view! {
                        <button
                            class="btn-loop btn-loop-bar"
                            // Grey out when no BPM is available.
                            class:loop-bar-disabled=move || bpm.get().is_none()
                            on:click=on_click
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{bar_loop_duration, clamp_loop_out};

    // ── bar_loop_duration ─────────────────────────────────────────────────────

    #[test]
    fn one_bar_at_120_bpm_is_two_seconds() {
        // 1 bar = 4 beats × (60 / 120) = 4 × 0.5 = 2.0 s
        let d = bar_loop_duration(1.0, 120.0);
        assert!((d - 2.0).abs() < 1e-9, "got {d}");
    }

    #[test]
    fn half_bar_at_120_bpm_is_one_second() {
        let d = bar_loop_duration(0.5, 120.0);
        assert!((d - 1.0).abs() < 1e-9, "got {d}");
    }

    #[test]
    fn four_bars_at_128_bpm() {
        // 4 bars × 4 beats × (60 / 128) ≈ 7.5 s
        let d = bar_loop_duration(4.0, 128.0);
        let expected = 4.0 * 4.0 * (60.0 / 128.0);
        assert!((d - expected).abs() < 1e-9, "got {d}");
    }

    #[test]
    fn bar_loop_duration_zero_bpm_returns_zero() {
        assert_eq!(bar_loop_duration(4.0, 0.0), 0.0);
    }

    #[test]
    fn bar_loop_duration_negative_bpm_returns_zero() {
        assert_eq!(bar_loop_duration(2.0, -90.0), 0.0);
    }

    // ── clamp_loop_out ────────────────────────────────────────────────────────

    #[test]
    fn clamp_loop_out_normal_case_passes_through() {
        // current (5.0) > loop_in (3.0) → no clamping needed
        assert!((clamp_loop_out(3.0, 5.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn clamp_loop_out_equal_to_in_nudges_forward() {
        let out = clamp_loop_out(4.0, 4.0);
        assert!(out > 4.0, "loop_out must be > loop_in; got {out}");
    }

    #[test]
    fn clamp_loop_out_before_in_nudges_forward() {
        // current (2.0) < loop_in (4.0) → must nudge to just past loop_in
        let out = clamp_loop_out(4.0, 2.0);
        assert!(out > 4.0, "loop_out must be > loop_in; got {out}");
    }

    // ── Loop-in invalidation logic ────────────────────────────────────────────

    /// Mirrors the deactivation guard in `on_loop_in`: if the new loop_in >= loop_out
    /// the region is invalid and the loop should be deactivated.
    fn should_deactivate_loop(new_loop_in: f64, loop_out: f64) -> bool {
        new_loop_in >= loop_out
    }

    #[test]
    fn loop_in_beyond_loop_out_should_deactivate() {
        assert!(should_deactivate_loop(25.0, 20.0));
    }

    #[test]
    fn loop_in_equal_to_loop_out_should_deactivate() {
        assert!(should_deactivate_loop(20.0, 20.0));
    }

    #[test]
    fn loop_in_before_loop_out_should_not_deactivate() {
        assert!(!should_deactivate_loop(10.0, 20.0));
    }
}
