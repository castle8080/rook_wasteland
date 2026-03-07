// Browser tests for the M9 randomize feature.
//
// All tests run in a real browser (headless Firefox) because `randomize` uses
// `js_sys::Math::random()` which is only available in WASM.
//
// Tests verify that every parameter is set to a value within its documented
// range and that the always-off invariants (posterize=0, invert=false) hold
// across multiple calls.
#![cfg(target_arch = "wasm32")]

use leptos::prelude::*;
use rw_teleidoscope::state::{randomize, KaleidoscopeParams};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Call `randomize` N times and return the resulting params each time.
fn run_n(n: usize) -> Vec<KaleidoscopeParams> {
    (0..n)
        .map(|_| {
            let p = KaleidoscopeParams::new();
            randomize(p);
            p
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Symmetry parameter ranges
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn segments_in_range() {
    for p in run_n(30) {
        let v = p.segments.get_untracked();
        assert!(
            (2..=10).contains(&v),
            "segments out of range: {v}"
        );
    }
}

#[wasm_bindgen_test]
fn rotation_in_range() {
    for p in run_n(30) {
        let v = p.rotation.get_untracked();
        assert!(
            (0.0..=360.0).contains(&v),
            "rotation out of range: {v}"
        );
    }
}

#[wasm_bindgen_test]
fn center_in_range() {
    for p in run_n(30) {
        let (cx, cy) = p.center.get_untracked();
        assert!(
            (0.2..0.8).contains(&cx),
            "center.x out of range: {cx}"
        );
        assert!(
            (0.2..0.8).contains(&cy),
            "center.y out of range: {cy}"
        );
    }
}

#[wasm_bindgen_test]
fn zoom_in_range() {
    for p in run_n(30) {
        let v = p.zoom.get_untracked();
        assert!(
            (0.3..=2.5).contains(&v),
            "zoom out of range: {v}"
        );
    }
}

// ---------------------------------------------------------------------------
// Effect constraints
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn at_least_one_effect_active() {
    // Over 30 runs, every result should have ≥ 1 effect > 0.
    for p in run_n(30) {
        let active = [
            p.spiral.get_untracked(),
            p.radial_fold.get_untracked(),
            p.lens.get_untracked(),
            p.ripple.get_untracked(),
        ]
        .iter()
        .filter(|&&v| v > 0.0)
        .count();
        assert!(active >= 1, "no effects active after randomize");
    }
}

#[wasm_bindgen_test]
fn at_most_two_effects_active() {
    for p in run_n(30) {
        let active = [
            p.spiral.get_untracked(),
            p.radial_fold.get_untracked(),
            p.lens.get_untracked(),
            p.ripple.get_untracked(),
        ]
        .iter()
        .filter(|&&v| v > 0.0)
        .count();
        assert!(active <= 2, "more than 2 effects active: {active}");
    }
}

#[wasm_bindgen_test]
fn active_effect_intensity_in_range() {
    for p in run_n(30) {
        for v in [
            p.spiral.get_untracked(),
            p.radial_fold.get_untracked(),
            p.lens.get_untracked(),
            p.ripple.get_untracked(),
        ] {
            if v > 0.0 {
                assert!(
                    (0.2..=1.0).contains(&v),
                    "active effect intensity out of range: {v}"
                );
            }
        }
    }
}

#[wasm_bindgen_test]
fn recursive_depth_in_range() {
    for p in run_n(30) {
        let v = p.recursive_depth.get_untracked();
        assert!(
            v <= 2,
            "recursive_depth out of range: {v}"
        );
    }
}

// ---------------------------------------------------------------------------
// Color transform ranges
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn hue_shift_in_range() {
    for p in run_n(30) {
        let v = p.hue_shift.get_untracked();
        assert!(
            (0.0..=360.0).contains(&v),
            "hue_shift out of range: {v}"
        );
    }
}

#[wasm_bindgen_test]
fn saturation_in_range() {
    for p in run_n(30) {
        let v = p.saturation.get_untracked();
        assert!(
            (0.8..=1.8).contains(&v),
            "saturation out of range: {v}"
        );
    }
}

#[wasm_bindgen_test]
fn brightness_in_range() {
    for p in run_n(30) {
        let v = p.brightness.get_untracked();
        assert!(
            (0.7..=1.5).contains(&v),
            "brightness out of range: {v}"
        );
    }
}

// ---------------------------------------------------------------------------
// Always-off invariants
// ---------------------------------------------------------------------------

#[wasm_bindgen_test]
fn posterize_always_off() {
    for p in run_n(30) {
        assert_eq!(
            p.posterize.get_untracked(),
            0,
            "posterize should always be 0 after randomize"
        );
    }
}

#[wasm_bindgen_test]
fn invert_always_off() {
    for p in run_n(30) {
        assert!(
            !p.invert.get_untracked(),
            "invert should always be false after randomize"
        );
    }
}
