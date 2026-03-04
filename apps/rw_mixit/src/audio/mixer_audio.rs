use std::f32::consts::PI;
use web_sys::{AudioContext, GainNode};

/// Shared Web Audio nodes for the mixer output stage.
///
/// Both decks route through this struct:
///   `AudioDeck.analyser → xfade_gain_[a|b] → master_gain → destination`
///
/// Created lazily once when the first `AudioDeck` is initialised.
pub struct MixerAudio {
    pub xfade_gain_a: GainNode,
    pub xfade_gain_b: GainNode,
    pub master_gain:  GainNode,
}

impl MixerAudio {
    /// Create the shared mixer output nodes and wire them to `ctx.destination`.
    ///
    /// Default crossfader is 0.5 (equal-power: both decks at ~0.707).
    /// The caller should apply current signal values immediately after
    /// construction if the user moved any sliders before loading a track.
    pub fn new(ctx: &AudioContext) -> Self {
        let xfade_gain_a = ctx.create_gain().expect("create_gain xfade_a");
        let xfade_gain_b = ctx.create_gain().expect("create_gain xfade_b");
        let master_gain  = ctx.create_gain().expect("create_gain master");

        let (cf_a, cf_b) = crossfader_gains(0.5);
        xfade_gain_a.gain().set_value(cf_a);
        xfade_gain_b.gain().set_value(cf_b);
        master_gain.gain().set_value(1.0);

        xfade_gain_a
            .connect_with_audio_node(&master_gain)
            .expect("connect xfade_a → master");
        xfade_gain_b
            .connect_with_audio_node(&master_gain)
            .expect("connect xfade_b → master");
        master_gain
            .connect_with_audio_node(&ctx.destination())
            .expect("connect master → destination");

        Self { xfade_gain_a, xfade_gain_b, master_gain }
    }

    /// Apply crossfader position using the equal-power cos/sin curve.
    /// `val` is in [0.0, 1.0]: 0 = full A, 0.5 = equal, 1 = full B.
    pub fn set_crossfader(&self, val: f32) {
        let (ga, gb) = crossfader_gains(val);
        self.xfade_gain_a.gain().set_value(ga);
        self.xfade_gain_b.gain().set_value(gb);
    }
}

/// Pure math for the equal-power crossfader curve.
///
/// Returns `(gain_a, gain_b)` for a crossfader position `val` ∈ [0.0, 1.0]:
/// - `val = 0.0` → `(1.0, 0.0)` — full Deck A, Deck B silent
/// - `val = 0.5` → `(~0.707, ~0.707)` — equal loudness both decks
/// - `val = 1.0` → `(0.0, 1.0)` — Deck A silent, full Deck B
///
/// The equal-power property guarantees `gain_a² + gain_b² = 1.0` at every
/// position, so perceived loudness stays constant as the fader is swept.
pub(crate) fn crossfader_gains(val: f32) -> (f32, f32) {
    let angle = val * PI / 2.0;
    (angle.cos(), angle.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Pure math tests (no browser required) ─────────────────────────

    #[test]
    fn crossfader_full_left_silences_b() {
        let (ga, gb) = crossfader_gains(0.0);
        assert!((ga - 1.0).abs() < 1e-6, "gain_a should be 1.0 at val=0, got {ga}");
        assert!(gb.abs() < 1e-6, "gain_b should be 0.0 at val=0, got {gb}");
    }

    #[test]
    fn crossfader_full_right_silences_a() {
        let (ga, gb) = crossfader_gains(1.0);
        assert!(ga.abs() < 1e-6, "gain_a should be 0.0 at val=1, got {ga}");
        assert!((gb - 1.0).abs() < 1e-6, "gain_b should be 1.0 at val=1, got {gb}");
    }

    #[test]
    fn crossfader_center_is_equal_power() {
        let (ga, gb) = crossfader_gains(0.5);
        let expected = (PI / 4.0_f32).cos(); // cos(45°) ≈ 0.7071
        assert!((ga - expected).abs() < 1e-5, "gain_a at 0.5 should be ~0.707, got {ga}");
        assert!((gb - expected).abs() < 1e-5, "gain_b at 0.5 should be ~0.707, got {gb}");
    }

    #[test]
    fn crossfader_constant_power_across_range() {
        // Equal-power: gain_a² + gain_b² = 1.0 for every position.
        for i in 0..=20 {
            let val = i as f32 / 20.0;
            let (ga, gb) = crossfader_gains(val);
            let power = ga * ga + gb * gb;
            assert!(
                (power - 1.0).abs() < 1e-5,
                "constant-power violated at val={val:.2}: ga²+gb² = {power}"
            );
        }
    }

    #[test]
    fn crossfader_gains_are_symmetric() {
        // crossfader_gains(x).0 == crossfader_gains(1-x).1  (A at x = B at mirror)
        for i in 0..=10 {
            let val = i as f32 / 10.0;
            let (ga, _) = crossfader_gains(val);
            let (_, gb_mirror) = crossfader_gains(1.0 - val);
            assert!(
                (ga - gb_mirror).abs() < 1e-5,
                "asymmetry at val={val:.1}: gain_a={ga}, mirrored gain_b={gb_mirror}"
            );
        }
    }

    // ── WASM tests (require a real browser / AudioContext) ────────────

    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;
        use wasm_bindgen_test::wasm_bindgen_test;
        use crate::audio::context::ensure_audio_context;

        fn make_ctx() -> AudioContext {
            let holder = std::rc::Rc::new(std::cell::RefCell::new(None::<AudioContext>));
            ensure_audio_context(&holder)
        }

        #[wasm_bindgen_test]
        fn mixer_audio_constructs_without_panic() {
            let _ma = MixerAudio::new(&make_ctx());
        }

        #[wasm_bindgen_test]
        fn default_xfade_a_gain_is_equal_power() {
            let ma = MixerAudio::new(&make_ctx());
            let ga = ma.xfade_gain_a.gain().value();
            let expected = (PI / 4.0_f32).cos();
            assert!(
                (ga - expected).abs() < 1e-5,
                "xfade_gain_a default should be ~0.707 (equal-power at 0.5), got {ga}"
            );
        }

        #[wasm_bindgen_test]
        fn default_xfade_b_gain_is_equal_power() {
            let ma = MixerAudio::new(&make_ctx());
            let gb = ma.xfade_gain_b.gain().value();
            let expected = (PI / 4.0_f32).sin();
            assert!(
                (gb - expected).abs() < 1e-5,
                "xfade_gain_b default should be ~0.707 (equal-power at 0.5), got {gb}"
            );
        }

        #[wasm_bindgen_test]
        fn default_master_gain_is_one() {
            let ma = MixerAudio::new(&make_ctx());
            let vol = ma.master_gain.gain().value();
            assert!((vol - 1.0).abs() < 1e-6, "master_gain default should be 1.0, got {vol}");
        }

        #[wasm_bindgen_test]
        fn set_crossfader_to_zero_silences_b() {
            let ma = MixerAudio::new(&make_ctx());
            ma.set_crossfader(0.0);
            let ga = ma.xfade_gain_a.gain().value();
            let gb = ma.xfade_gain_b.gain().value();
            assert!((ga - 1.0).abs() < 1e-5, "gain_a should be 1.0 after set_crossfader(0), got {ga}");
            assert!(gb.abs() < 1e-5, "gain_b should be 0.0 after set_crossfader(0), got {gb}");
        }

        #[wasm_bindgen_test]
        fn set_crossfader_to_one_silences_a() {
            let ma = MixerAudio::new(&make_ctx());
            ma.set_crossfader(1.0);
            let ga = ma.xfade_gain_a.gain().value();
            let gb = ma.xfade_gain_b.gain().value();
            assert!(ga.abs() < 1e-5, "gain_a should be 0.0 after set_crossfader(1), got {ga}");
            assert!((gb - 1.0).abs() < 1e-5, "gain_b should be 1.0 after set_crossfader(1), got {gb}");
        }

        #[wasm_bindgen_test]
        fn connect_to_mixer_output_does_not_panic() {
            let ctx = make_ctx();
            let ma   = MixerAudio::new(&ctx);
            let deck = crate::audio::deck_audio::AudioDeck::new(ctx);
            // Should connect analyser → xfade_gain_a without panicking.
            deck.borrow().connect_to_mixer_output(&ma.xfade_gain_a);
        }
    }
}
