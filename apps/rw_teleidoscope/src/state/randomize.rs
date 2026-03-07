use leptos::prelude::*;

use super::KaleidoscopeParams;

/// Randomise all [`KaleidoscopeParams`] signals to visually interesting values.
///
/// Uses `js_sys::Math::random()` (WASM-only) to generate uniform random
/// numbers.  All parameter values are clamped to ranges that produce
/// recognisable, non-degenerate kaleidoscope output.
///
/// **Ranges applied:**
/// - `segments`: integer 2–10
/// - `rotation`: f32 0.0–360.0°
/// - `center`: (f32, f32) both in 0.2–0.8 (avoids extreme edges)
/// - `zoom`: f32 0.3–2.5
/// - effects: 1 or 2 of {spiral, radial_fold, lens, ripple} at intensity
///   0.2–1.0; the rest are set to 0.0
/// - `mobius`: `true` with 30 % probability
/// - `recursive_depth`: 0 (60 % chance), 1 (20 %), 2 (20 %)
/// - `hue_shift`: f32 0.0–360.0
/// - `saturation`: f32 0.8–1.8
/// - `brightness`: f32 0.7–1.5
/// - `posterize`: always 0 (off)
/// - `invert`: always `false`
pub fn randomize(params: KaleidoscopeParams) {
    /// Return a random f32 in the half-open interval [lo, hi).
    fn rng(lo: f32, hi: f32) -> f32 {
        lo + (js_sys::Math::random() as f32) * (hi - lo)
    }

    // --- Symmetry -----------------------------------------------------------

    // random() * 9.0 ∈ [0.0, 9.0) → truncate to u32 ∈ [0, 8] → add 2 → [2, 10]
    params.segments.set(2 + (js_sys::Math::random() * 9.0) as u32);
    params.rotation.set(rng(0.0, 360.0));
    params.center.set((rng(0.2, 0.8), rng(0.2, 0.8)));
    params.zoom.set(rng(0.3, 2.5));

    // --- Effects ------------------------------------------------------------

    // Zero all float effects first, then activate 1 or 2 at random intensity.
    params.spiral.set(0.0);
    params.radial_fold.set(0.0);
    params.lens.set(0.0);
    params.ripple.set(0.0);

    // Pick 1 or 2 effects with equal probability.
    let count: usize = if js_sys::Math::random() < 0.5 { 1 } else { 2 };

    // Partial Fisher-Yates shuffle on an index array [0, 1, 2, 3].
    // After the loop the first `count` elements are a distinct random sample.
    let mut indices = [0usize, 1, 2, 3];
    for i in 0..count {
        // j is in [i, 3] inclusive — never out of bounds.
        let remaining = 4 - i;
        let offset = (js_sys::Math::random() * remaining as f64) as usize;
        let j = i + offset.min(remaining - 1);
        indices.swap(i, j);
    }

    for &idx in indices.iter().take(count) {
        let intensity = rng(0.2, 1.0);
        match idx {
            0 => params.spiral.set(intensity),
            1 => params.radial_fold.set(intensity),
            2 => params.lens.set(intensity),
            _ => params.ripple.set(intensity),
        }
    }

    // Möbius: enabled with 30 % probability.
    params.mobius.set(js_sys::Math::random() < 0.3);

    // Recursive depth: 0 (60 %), 1 (20 %), 2 (20 %).
    let depth: u32 = if js_sys::Math::random() < 0.6 {
        0
    } else if js_sys::Math::random() < 0.5 {
        1
    } else {
        2
    };
    params.recursive_depth.set(depth);

    // --- Color transforms ---------------------------------------------------

    params.hue_shift.set(rng(0.0, 360.0));
    params.saturation.set(rng(0.8, 1.8));
    params.brightness.set(rng(0.7, 1.5));

    // Posterize and invert are excluded — they produce extreme / broken-looking
    // output when set to random values.
    params.posterize.set(0);
    params.invert.set(false);
}
