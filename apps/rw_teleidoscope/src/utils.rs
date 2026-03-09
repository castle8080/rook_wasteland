//! Shared utility helpers.

use wasm_bindgen::JsCast;

/// Accepted image MIME types for the file-loading pipeline.
pub const ACCEPTED_IMAGE_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp"];

/// Returns `true` if `mime` is one of the image types the app can process.
///
/// Extracted from the UI layer so the acceptance list can be tested without
/// a browser or a real `File` object.
pub fn is_accepted_image_type(mime: &str) -> bool {
    ACCEPTED_IMAGE_TYPES.contains(&mime)
}

/// Pure Rust equivalent of the GLSL barrel-warp (lens distortion) formula.
///
/// Matches the shader expression `r = r / max(1.0 - lens * r * r, 0.001)`.
/// Extracted here so the formula can be unit-tested on the native target without
/// a WebGL context.
///
/// At `lens == 0.0` this is the identity (`r / 1.0 == r`).
pub fn lens_warp(r: f32, lens: f32) -> f32 {
    let denom = (1.0 - lens * r * r).max(0.001);
    r / denom
}

/// Pure Rust equivalent of the GLSL radial-fold formula.
///
/// Matches the shader expression
/// `r = abs(mod(r * (1.0 + fold * 4.0), 2.0) - 1.0)`.
/// Extracted here so the formula can be unit-tested on the native target.
///
/// Note: this formula is **not** an identity at `fold == 0.0`
/// (`abs(mod(r, 2.0) - 1.0)` is not `r` for all r).  The shader therefore
/// gates application behind `if (u_radial_fold > 0.0)`.
pub fn radial_fold_r(r: f32, fold: f32) -> f32 {
    ((r * (1.0 + fold * 4.0)).rem_euclid(2.0) - 1.0).abs()
}
///
/// Replicates the GLSL fold in the fragment shader so the algorithm can be
/// unit-tested on the native target without a WebGL context.
///
/// Uses `rem_euclid` (floor-based modulo) to match GLSL's `mod()` behaviour for
/// negative angles.
pub fn mirror_fold(a: f32, segments: u32) -> f32 {
    let seg_angle = std::f32::consts::PI / segments as f32;
    let two_seg = 2.0 * seg_angle;
    let a = a.rem_euclid(two_seg);
    if a > seg_angle {
        two_seg - a
    } else {
        a
    }
}

/// Pure Rust equivalent of the GLSL RGB→HSV→RGB hue-rotation used in `frag.glsl`.
///
/// Converts the input RGB colour to HSV, adds `degrees` to the hue (mod 360),
/// then converts back.  Returns the result as an `(r, g, b)` tuple with all
/// channels in [0, 1].
///
/// At `degrees == 0.0` the output equals the input.
pub fn hue_rotate_rgb(r: f32, g: f32, b: f32, degrees: f32) -> (f32, f32, f32) {
    // RGB → HSV
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    let delta = cmax - cmin;

    let h = if delta < f32::EPSILON {
        0.0_f32
    } else if (cmax - r).abs() < f32::EPSILON {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if (cmax - g).abs() < f32::EPSILON {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };

    let s = if cmax > f32::EPSILON { delta / cmax } else { 0.0 };
    let v = cmax;

    // Rotate hue
    let h = (h + degrees).rem_euclid(360.0);

    // HSV → RGB
    let f = (h / 60.0).rem_euclid(6.0);
    let i = f.floor();
    let ff = f - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * ff);
    let t = v * (1.0 - s * (1.0 - ff));

    let (ro, go, bo) = match i as u32 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q), // i == 5
    };
    (ro, go, bo)
}

/// Pure Rust equivalent of the GLSL `posterize` formula used in `frag.glsl`.
///
/// Clamps `v` to [0, 1] then quantises it to `levels` discrete steps:
/// `floor(clamp(v, 0, 1) * levels) / levels`.
///
/// `levels == 0` is safe (returns 0.0); callers should gate on `levels > 1`
/// to match the `if (u_posterize > 1)` shader guard.
pub fn posterize_channel(v: f32, levels: u32) -> f32 {
    if levels == 0 {
        return 0.0;
    }
    let lev = levels as f32;
    (v.clamp(0.0, 1.0) * lev).floor() / lev
}

const RESIZE_TARGET: u32 = 800;

/// Compute the `(dx, dy, draw_w, draw_h)` parametersthat cover-scale an image
/// of `(img_w × img_h)` pixels into a square canvas of `target` pixels.
///
/// The image is scaled uniformly until it fills the entire target square, then
/// centred.  Any overflow outside the square is cropped by the canvas clipping.
/// This is the "cover" algorithm used by `background-size: cover` in CSS.
pub fn cover_rect(img_w: f64, img_h: f64, target: f64) -> (f64, f64, f64, f64) {
    let scale = (target / img_w).max(target / img_h);
    let draw_w = img_w * scale;
    let draw_h = img_h * scale;
    let dx = (target - draw_w) / 2.0;
    let dy = (target - draw_h) / 2.0;
    (dx, dy, draw_w, draw_h)
}

/// Draw `image` onto an offscreen 800 × 800 `<canvas>` (cover-scaled, centre-
/// cropped) and return the resulting `ImageData` (always 800 × 800 RGBA).
///
/// The offscreen canvas is **not** added to the DOM.  `image` must have already
/// fully loaded before this is called (i.e. the `onload` event has fired);
/// otherwise `naturalWidth` / `naturalHeight` are zero and the result is blank.
pub fn resize_to_800(image: &web_sys::HtmlImageElement) -> Result<web_sys::ImageData, String> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let canvas: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|e| format!("create canvas: {e:?}"))?
        .dyn_into()
        .map_err(|_| "element is not HtmlCanvasElement".to_string())?;

    let target = f64::from(RESIZE_TARGET);
    canvas.set_width(RESIZE_TARGET);
    canvas.set_height(RESIZE_TARGET);

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|e| format!("get_context failed: {e:?}"))?
        .ok_or("no 2d context")?
        .dyn_into()
        .map_err(|_| "not a CanvasRenderingContext2d".to_string())?;

    let img_w = f64::from(image.natural_width());
    let img_h = f64::from(image.natural_height());
    let (dx, dy, draw_w, draw_h) = cover_rect(img_w, img_h, target);

    ctx.draw_image_with_html_image_element_and_dw_and_dh(image, dx, dy, draw_w, draw_h)
        .map_err(|e| format!("drawImage failed: {e:?}"))?;

    ctx.get_image_data(0.0, 0.0, target, target)
        .map_err(|e| format!("getImageData failed: {e:?}"))
}

/// Euclidean distance between two 2-D points in the same coordinate space.
///
/// Used to compute the pinch distance between two active touch-pointer positions
/// before and after a move, so that `apply_pinch_zoom` can derive the zoom delta.
pub fn pinch_distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    (dx * dx + dy * dy).sqrt()
}

/// Apply a pinch-zoom gesture delta to `current_zoom` and clamp to [`min`, `max`].
///
/// The new zoom is `(new_dist / old_dist) * current_zoom`.  If `old_dist` ≤ 0
/// (the two fingers were at the same pixel, or only one finger is active) the
/// function returns `current_zoom` unchanged to avoid a divide-by-zero.
pub fn apply_pinch_zoom(
    old_dist: f32,
    new_dist: f32,
    current_zoom: f32,
    min: f32,
    max: f32,
) -> f32 {
    if old_dist <= 0.0 {
        return current_zoom;
    }
    (new_dist / old_dist * current_zoom).clamp(min, max)
}

#[cfg(test)]
mod tests {
    use super::{apply_pinch_zoom, cover_rect, hue_rotate_rgb, is_accepted_image_type, lens_warp,
                mirror_fold, pinch_distance, posterize_channel, radial_fold_r};

    #[test]
    fn cover_rect_square_image_is_identity() {
        let (dx, dy, draw_w, draw_h) = cover_rect(800.0, 800.0, 800.0);
        assert!((dx - 0.0).abs() < 1e-9);
        assert!((dy - 0.0).abs() < 1e-9);
        assert!((draw_w - 800.0).abs() < 1e-9);
        assert!((draw_h - 800.0).abs() < 1e-9);
    }

    #[test]
    fn cover_rect_wide_image_clips_horizontally() {
        // 1600×800 → scale = max(800/1600, 800/800) = max(0.5, 1.0) = 1.0
        // draw_w = 1600, draw_h = 800, dx = -400, dy = 0
        let (dx, dy, draw_w, draw_h) = cover_rect(1600.0, 800.0, 800.0);
        assert!((dy - 0.0).abs() < 1e-9, "dy should be 0, got {dy}");
        assert!((draw_h - 800.0).abs() < 1e-9);
        assert!((draw_w - 1600.0).abs() < 1e-9);
        assert!((dx - (-400.0)).abs() < 1e-9);
    }

    #[test]
    fn cover_rect_tall_image_clips_vertically() {
        // 800×1600 → scale = max(800/800, 800/1600) = max(1.0, 0.5) = 1.0
        // draw_w = 800, draw_h = 1600, dx = 0, dy = -400
        let (dx, dy, draw_w, draw_h) = cover_rect(800.0, 1600.0, 800.0);
        assert!((dx - 0.0).abs() < 1e-9);
        assert!((draw_w - 800.0).abs() < 1e-9);
        assert!((draw_h - 1600.0).abs() < 1e-9);
        assert!((dy - (-400.0)).abs() < 1e-9);
    }

    #[test]
    fn cover_rect_small_image_scales_up() {
        // 100×100 → scale = 8.0
        let (dx, dy, draw_w, draw_h) = cover_rect(100.0, 100.0, 800.0);
        assert!((dx - 0.0).abs() < 1e-9);
        assert!((dy - 0.0).abs() < 1e-9);
        assert!((draw_w - 800.0).abs() < 1e-9);
        assert!((draw_h - 800.0).abs() < 1e-9);
    }

    #[test]
    fn cover_rect_4_3_photo_ratio() {
        // 1200×900 (common 4:3 photo) → scale = max(800/1200, 800/900) ≈ max(0.666, 0.888) = 0.888…
        // draw_h = 800, draw_w = 1200 * (800/900) ≈ 1066.66, dx ≈ -133.33, dy = 0
        let (dx, dy, draw_w, draw_h) = cover_rect(1200.0, 900.0, 800.0);
        let scale = 800.0_f64 / 900.0;
        assert!((dy - 0.0).abs() < 1e-9, "dy should be 0, got {dy}");
        assert!((draw_h - 800.0).abs() < 1e-9);
        assert!((draw_w - 1200.0 * scale).abs() < 1e-9);
        assert!((dx - (800.0 - 1200.0 * scale) / 2.0).abs() < 1e-9);
    }

    // -- mirror_fold tests (pure math, no browser needed) --------------------

    #[test]
    fn mirror_fold_zero_stays_zero() {
        assert!((mirror_fold(0.0, 6) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn mirror_fold_at_seg_angle_is_identity() {
        let seg_angle = std::f32::consts::PI / 6.0;
        assert!((mirror_fold(seg_angle, 6) - seg_angle).abs() < 1e-6);
    }

    #[test]
    fn mirror_fold_at_two_seg_angle_wraps_to_zero() {
        // rem_euclid(2*seg_angle, 2*seg_angle) = 0.0
        let seg_angle = std::f32::consts::PI / 6.0;
        let result = mirror_fold(2.0 * seg_angle, 6);
        assert!(result < 1e-5, "expected ~0, got {result}");
    }

    #[test]
    fn mirror_fold_above_seg_angle_folds_back() {
        // a = 1.5 * seg_angle is in the second half → folds to 0.5 * seg_angle
        let seg_angle = std::f32::consts::PI / 6.0;
        let a = seg_angle * 1.5;
        let expected = 2.0 * seg_angle - a; // = 0.5 * seg_angle
        assert!((mirror_fold(a, 6) - expected).abs() < 1e-6);
    }

    #[test]
    fn mirror_fold_negative_angle_wraps_and_folds() {
        // rem_euclid(-0.1, 2*seg_angle) = 2*seg_angle - 0.1  (> seg_angle)
        // fold: 2*seg_angle - (2*seg_angle - 0.1) = 0.1
        let result = mirror_fold(-0.1, 6);
        assert!((result - 0.1).abs() < 1e-5, "expected ~0.1, got {result}");
    }

    #[test]
    fn mirror_fold_segments_2_boundary_values() {
        let seg_angle = std::f32::consts::PI / 2.0;
        assert!((mirror_fold(0.0, 2) - 0.0).abs() < 1e-6);
        assert!((mirror_fold(seg_angle, 2) - seg_angle).abs() < 1e-6);
    }

    #[test]
    fn mirror_fold_segments_10_boundary_values() {
        let seg_angle = std::f32::consts::PI / 10.0;
        assert!((mirror_fold(0.0, 10) - 0.0).abs() < 1e-6);
        assert!((mirror_fold(seg_angle, 10) - seg_angle).abs() < 1e-6);
    }


    #[test]
    fn accepted_mime_types_allow_png_jpeg_webp() {
        assert!(is_accepted_image_type("image/png"));
        assert!(is_accepted_image_type("image/jpeg"));
        assert!(is_accepted_image_type("image/webp"));
    }

    #[test]
    fn accepted_mime_types_reject_gif_and_others() {
        assert!(!is_accepted_image_type("image/gif"));
        assert!(!is_accepted_image_type("image/bmp"));
        assert!(!is_accepted_image_type("image/svg+xml"));
        assert!(!is_accepted_image_type("text/plain"));
        assert!(!is_accepted_image_type(""));
    }

    // -- lens_warp tests (pure math, no browser needed) ----------------------

    #[test]
    fn lens_warp_zero_r_is_always_zero() {
        assert!((lens_warp(0.0, 0.0) - 0.0).abs() < 1e-6);
        assert!((lens_warp(0.0, 0.5) - 0.0).abs() < 1e-6);
        assert!((lens_warp(0.0, 1.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn lens_warp_zero_lens_is_identity() {
        // lens=0 → denom = max(1-0, 0.001) = 1.0 → r unchanged
        assert!((lens_warp(0.5, 0.0) - 0.5).abs() < 1e-6);
        assert!((lens_warp(1.0, 0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn lens_warp_mid_values() {
        // r=0.5, lens=0.8 → denom = max(1 - 0.8*0.25, 0.001) = 0.8
        // r_new = 0.5 / 0.8 = 0.625
        let result = lens_warp(0.5, 0.8);
        assert!((result - 0.625).abs() < 1e-5, "got {result}");
    }

    #[test]
    fn lens_warp_denominator_clamped() {
        // r=1.0, lens=1.0 → denom = max(1 - 1*1, 0.001) = 0.001
        // r_new = 1.0 / 0.001 = 1000.0
        let result = lens_warp(1.0, 1.0);
        assert!((result - 1000.0).abs() < 0.1, "got {result}");
    }

    // -- radial_fold_r tests (pure math, no browser needed) ------------------

    #[test]
    fn radial_fold_r_fold_1_mid_radius() {
        // r=0.5, fold=1 → r_new = abs(mod(0.5*(1+4), 2) - 1)
        //                       = abs(mod(2.5, 2) - 1) = abs(0.5 - 1) = 0.5
        let result = radial_fold_r(0.5, 1.0);
        assert!((result - 0.5).abs() < 1e-5, "got {result}");
    }

    #[test]
    fn radial_fold_r_fold_half_small_radius() {
        // r=0.1, fold=0.5 → r_new = abs(mod(0.1*(1+2), 2) - 1)
        //                          = abs(mod(0.3, 2) - 1) = abs(0.3 - 1) = 0.7
        let result = radial_fold_r(0.1, 0.5);
        assert!((result - 0.7).abs() < 1e-5, "got {result}");
    }

    #[test]
    fn radial_fold_r_result_is_always_in_zero_one_range() {
        // abs(mod(x, 2) - 1) is always in [0, 1]
        for &r in &[0.0_f32, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0] {
            for &fold in &[0.0_f32, 0.25, 0.5, 0.75, 1.0] {
                let result = radial_fold_r(r, fold);
                assert!(
                    (0.0..=1.0 + 1e-5_f32).contains(&result),
                    "radial_fold_r({r}, {fold}) = {result} is out of [0,1]"
                );
            }
        }
    }

    // -- hue_rotate_rgb tests (pure math, no browser needed) -----------------

    #[test]
    fn hue_rotate_rgb_zero_degrees_is_identity() {
        let (r, g, b) = hue_rotate_rgb(0.8, 0.4, 0.2, 0.0);
        assert!((r - 0.8).abs() < 1e-5, "r: {r}");
        assert!((g - 0.4).abs() < 1e-5, "g: {g}");
        assert!((b - 0.2).abs() < 1e-5, "b: {b}");
    }

    #[test]
    fn hue_rotate_rgb_360_degrees_wraps_to_identity() {
        let (r0, g0, b0) = hue_rotate_rgb(0.6, 0.3, 0.9, 0.0);
        let (r1, g1, b1) = hue_rotate_rgb(0.6, 0.3, 0.9, 360.0);
        assert!((r0 - r1).abs() < 1e-5, "r: {r0} vs {r1}");
        assert!((g0 - g1).abs() < 1e-5, "g: {g0} vs {g1}");
        assert!((b0 - b1).abs() < 1e-5, "b: {b0} vs {b1}");
    }

    #[test]
    fn hue_rotate_rgb_180_shifts_hue_by_half_spectrum() {
        // Pure red (1,0,0) rotated 180° should become cyan (0,1,1).
        let (r, g, b) = hue_rotate_rgb(1.0, 0.0, 0.0, 180.0);
        assert!((r - 0.0).abs() < 1e-4, "r should be ~0, got {r}");
        assert!((g - 1.0).abs() < 1e-4, "g should be ~1, got {g}");
        assert!((b - 1.0).abs() < 1e-4, "b should be ~1, got {b}");
    }

    #[test]
    fn hue_rotate_rgb_120_red_becomes_green() {
        // Pure red (1,0,0) rotated 120° should become pure green (0,1,0).
        let (r, g, b) = hue_rotate_rgb(1.0, 0.0, 0.0, 120.0);
        assert!((r - 0.0).abs() < 1e-4, "r should be ~0, got {r}");
        assert!((g - 1.0).abs() < 1e-4, "g should be ~1, got {g}");
        assert!((b - 0.0).abs() < 1e-4, "b should be ~0, got {b}");
    }

    #[test]
    fn hue_rotate_rgb_grey_is_unchanged() {
        // Grey (0.5, 0.5, 0.5) has zero saturation; hue rotation should be a no-op.
        let (r, g, b) = hue_rotate_rgb(0.5, 0.5, 0.5, 90.0);
        assert!((r - 0.5).abs() < 1e-5, "r: {r}");
        assert!((g - 0.5).abs() < 1e-5, "g: {g}");
        assert!((b - 0.5).abs() < 1e-5, "b: {b}");
    }

    // -- posterize_channel tests (pure math, no browser needed) --------------

    #[test]
    fn posterize_channel_2_levels_quantises_correctly() {
        // levels=2 → steps at 0.0 and 0.5
        assert!((posterize_channel(0.0, 2) - 0.0).abs() < 1e-6);
        assert!((posterize_channel(0.4, 2) - 0.0).abs() < 1e-6, "0.4 → 0.0");
        assert!((posterize_channel(0.5, 2) - 0.5).abs() < 1e-6, "0.5 → 0.5");
        assert!((posterize_channel(0.9, 2) - 0.5).abs() < 1e-6, "0.9 → 0.5");
        assert!((posterize_channel(1.0, 2) - 1.0).abs() < 1e-6, "1.0 → 1.0");
    }

    #[test]
    fn posterize_channel_4_levels() {
        // levels=4 → steps at 0.0, 0.25, 0.5, 0.75
        assert!((posterize_channel(0.1, 4) - 0.0).abs() < 1e-6, "0.1");
        assert!((posterize_channel(0.3, 4) - 0.25).abs() < 1e-6, "0.3");
        assert!((posterize_channel(0.6, 4) - 0.5).abs() < 1e-6, "0.6");
        assert!((posterize_channel(0.8, 4) - 0.75).abs() < 1e-6, "0.8");
    }

    #[test]
    fn posterize_channel_clamps_above_1() {
        // Values > 1.0 (HDR overshoot) should be clamped before quantising.
        let result = posterize_channel(1.5, 4);
        assert!((result - 1.0).abs() < 1e-6, "expected 1.0, got {result}");
    }

    #[test]
    fn posterize_channel_clamps_below_0() {
        let result = posterize_channel(-0.5, 4);
        assert!((result - 0.0).abs() < 1e-6, "expected 0.0, got {result}");
    }

    #[test]
    fn posterize_channel_zero_levels_returns_zero() {
        // levels=0 guard — should not divide by zero.
        assert!((posterize_channel(0.7, 0) - 0.0).abs() < 1e-6);
    }

    // -- pinch_distance tests (pure math, no browser needed) -----------------

    #[test]
    fn pinch_distance_same_point_is_zero() {
        assert!((pinch_distance(3.0, 4.0, 3.0, 4.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn pinch_distance_horizontal() {
        assert!((pinch_distance(0.0, 0.0, 100.0, 0.0) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn pinch_distance_vertical() {
        assert!((pinch_distance(0.0, 0.0, 0.0, 200.0) - 200.0).abs() < 1e-4);
    }

    #[test]
    fn pinch_distance_3_4_5_triangle() {
        assert!((pinch_distance(0.0, 0.0, 3.0, 4.0) - 5.0).abs() < 1e-5);
    }

    // -- apply_pinch_zoom tests (pure math, no browser needed) ----------------

    #[test]
    fn apply_pinch_zoom_scale_out() {
        // Fingers move apart: new_dist > old_dist → zoom increases.
        let result = apply_pinch_zoom(100.0, 200.0, 1.0, 0.5, 4.0);
        assert!((result - 2.0).abs() < 1e-5, "got {result}");
    }

    #[test]
    fn apply_pinch_zoom_scale_in() {
        // Fingers move together: new_dist < old_dist → zoom decreases.
        let result = apply_pinch_zoom(200.0, 100.0, 2.0, 0.5, 4.0);
        assert!((result - 1.0).abs() < 1e-5, "got {result}");
    }

    #[test]
    fn apply_pinch_zoom_clamps_at_max() {
        let result = apply_pinch_zoom(100.0, 1000.0, 3.0, 0.5, 4.0);
        assert!((result - 4.0).abs() < 1e-5, "expected max 4.0, got {result}");
    }

    #[test]
    fn apply_pinch_zoom_clamps_at_min() {
        let result = apply_pinch_zoom(1000.0, 10.0, 1.0, 0.5, 4.0);
        assert!((result - 0.5).abs() < 1e-5, "expected min 0.5, got {result}");
    }

    #[test]
    fn apply_pinch_zoom_zero_old_dist_returns_unchanged() {
        let result = apply_pinch_zoom(0.0, 100.0, 1.5, 0.5, 4.0);
        assert!((result - 1.5).abs() < 1e-6, "expected 1.5 unchanged, got {result}");
    }
}

