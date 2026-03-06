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

/// Mirror-fold angle `a` (radians) into the fundamental domain `[0, π/segments]`.
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

#[cfg(test)]
mod tests {
    use super::{cover_rect, is_accepted_image_type, mirror_fold};

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
}

