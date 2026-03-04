/// Pitch / tempo fader component and supporting conversion functions.
///
/// The fader is a horizontal `<input type="range">` ranging from −1.0 to +1.0.
/// It writes to `DeckState.playback_rate` via `pitch_to_rate()` on every input
/// event.  A companion `rate_to_pitch()` is provided for the inverse conversion
/// (initialising the slider position from the current rate).
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::state::DeckState;

// ── Pure conversion functions ─────────────────────────────────────────────────

/// Convert a pitch fader position (−1.0 to +1.0) to a playback-rate multiplier.
///
/// Vinyl mode only — pitch changes with speed, exactly as on a real record:
/// - Fader at  0.0 → rate 1.0 (no change)
/// - Fader at +1.0 → rate 2.0 (+100 %, double speed / pitch)
/// - Fader at −1.0 → rate 0.5 (−50 %, half speed / pitch)
///
/// The asymmetric formula `1 / (1 − fader)` for negative values ensures that
/// equal fader distances below zero produce the same perceived pitch interval
/// as above zero (octave-symmetric).
pub fn pitch_to_rate(fader: f64) -> f64 {
    if fader >= 0.0 {
        1.0 + fader
    } else {
        1.0 / (1.0 - fader)
    }
}

/// Convert a playback rate back to a pitch fader position.
///
/// Exact inverse of `pitch_to_rate`; used to initialise the slider from the
/// current `DeckState.playback_rate` value.
pub fn rate_to_pitch(rate: f64) -> f64 {
    if rate >= 1.0 {
        rate - 1.0
    } else {
        1.0 - 1.0 / rate
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Horizontal pitch / tempo fader (−50 % to +50 %).
///
/// Moving the slider updates `state.playback_rate` in real time.
/// A small readout below shows the current percentage deviation.
#[component]
pub fn PitchFader(state: DeckState) -> impl IntoView {
    // Initialise fader position from the current playback_rate.
    let fader_val = RwSignal::new(rate_to_pitch(state.playback_rate.get_untracked()));

    let on_input = {
        let state = state.clone();
        move |ev: web_sys::Event| {
            let input: web_sys::HtmlInputElement = ev
                .target()
                .expect("PitchFader — event target")
                .unchecked_into();
            let raw = input.value_as_number();
            let clamped = raw.clamp(-1.0, 1.0);
            fader_val.set(clamped);
            state.playback_rate.set(pitch_to_rate(clamped));
        }
    };

    view! {
        <div class="pitch-fader-wrapper">
            <div class="pitch-fader-label">"PITCH"</div>
            <div class="pitch-fader-track">
                <span class="pitch-fader-limit">"-50%"</span>
                <input
                    class="pitch-fader"
                    type="range"
                    min="-1.0"
                    max="1.0"
                    step="0.01"
                    prop:value=move || fader_val.get()
                    on:input=on_input
                />
                <span class="pitch-fader-limit">"+50%"</span>
            </div>
            <div class="pitch-fader-readout">
                {move || {
                    let rate = state.playback_rate.get();
                    let pct = (rate - 1.0) * 100.0;
                    if pct.abs() < 0.05 {
                        "±0.0%".to_string()
                    } else if pct > 0.0 {
                        format!("+{:.1}%", pct)
                    } else {
                        format!("{:.1}%", pct)
                    }
                }}
            </div>
        </div>
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_to_rate_at_zero_is_one() {
        assert!((pitch_to_rate(0.0) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn pitch_to_rate_at_plus_one_is_two() {
        assert!((pitch_to_rate(1.0) - 2.0).abs() < 1e-12);
    }

    #[test]
    fn pitch_to_rate_at_minus_one_is_half() {
        assert!((pitch_to_rate(-1.0) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn pitch_to_rate_monotone_positive() {
        // Increasing fader above 0 should always increase rate.
        assert!(pitch_to_rate(0.5) > pitch_to_rate(0.25));
        assert!(pitch_to_rate(0.25) > pitch_to_rate(0.0));
    }

    #[test]
    fn pitch_to_rate_monotone_negative() {
        // Decreasing fader below 0 should always decrease rate.
        assert!(pitch_to_rate(-0.25) < pitch_to_rate(0.0));
        assert!(pitch_to_rate(-0.5) < pitch_to_rate(-0.25));
    }

    #[test]
    fn rate_to_pitch_roundtrip() {
        for &fader in &[-1.0_f64, -0.5, -0.25, 0.0, 0.25, 0.5, 1.0] {
            let rate = pitch_to_rate(fader);
            let back = rate_to_pitch(rate);
            assert!(
                (back - fader).abs() < 1e-9,
                "roundtrip failed: fader={fader} → rate={rate} → {back}"
            );
        }
    }

    #[test]
    fn rate_to_pitch_at_rate_one_is_zero() {
        assert!(rate_to_pitch(1.0).abs() < 1e-12);
    }
}
