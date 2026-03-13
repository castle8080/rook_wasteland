/// Viewport-scale utility — computes and applies a CSS transform scale so that
/// the full DJ mixer UI fits within small viewports without scrolling.
///
/// Strategy: CSS `@media (max-height)` breakpoints shrink canvas elements and
/// reduce padding progressively.  When the viewport drops below
/// [`SCALE_THRESHOLD_PX`], a `transform: scale(factor)` is applied to
/// `#rw-mixit-root` via the `--app-scale` CSS custom property, providing a
/// final safety net for extreme sizes (e.g. 800×600 with browser chrome).
use wasm_bindgen::JsCast;

/// Viewport height (in CSS px) below which the CSS-transform scale fallback
/// activates.  Above this value the CSS breakpoints are sufficient; below it
/// the entire root element is scaled down proportionally.
pub const SCALE_THRESHOLD_PX: f64 = 640.0;

/// Floor for the computed scale factor.  Prevents the UI from becoming too
/// small to interact with on extreme window sizes.
pub const MIN_SCALE: f64 = 0.6;

/// Compute the `transform: scale()` factor for a given viewport height.
///
/// Returns `1.0` when `viewport_height >= SCALE_THRESHOLD_PX` (no scaling
/// needed); proportionally less below the threshold; clamped to
/// [`MIN_SCALE`] at the bottom.
///
/// Extracted as a pure function so it can be covered by native `cargo test`
/// without a browser.
pub fn compute_scale_factor(viewport_height: f64) -> f64 {
    if viewport_height < SCALE_THRESHOLD_PX {
        (viewport_height / SCALE_THRESHOLD_PX).max(MIN_SCALE)
    } else {
        1.0
    }
}

/// Read `window.innerHeight`, compute the scale factor via
/// [`compute_scale_factor`], and write the result as `--app-scale` on
/// `document.documentElement.style`.
///
/// The CSS rule `#rw-mixit-root { transform: scale(var(--app-scale, 1.0)) }`
/// picks up this property on the next paint.  This function is a no-op when
/// `window` or `document` are unavailable (e.g. during `cargo test`).
pub fn update_viewport_scale() {
    let Some(win) = web_sys::window() else { return };

    let height = win
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(900.0); // fallback = baseline, no scale applied

    let scale = compute_scale_factor(height);

    let Some(doc) = win.document() else { return };
    let Some(root_el) = doc.document_element() else { return };
    let style = root_el
        .unchecked_into::<web_sys::HtmlElement>()
        .style();
    let _ = style.set_property("--app-scale", &format!("{scale:.4}"));
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_at_threshold_is_one() {
        assert_eq!(compute_scale_factor(SCALE_THRESHOLD_PX), 1.0);
    }

    #[test]
    fn scale_above_threshold_is_one() {
        assert_eq!(compute_scale_factor(SCALE_THRESHOLD_PX + 100.0), 1.0);
    }

    #[test]
    fn scale_below_threshold_is_proportional() {
        let h = 480.0;
        let expected = h / SCALE_THRESHOLD_PX;
        let got = compute_scale_factor(h);
        assert!((got - expected).abs() < 1e-10, "expected {expected}, got {got}");
    }

    #[test]
    fn scale_at_extreme_low_is_clamped_to_min() {
        assert_eq!(compute_scale_factor(100.0), MIN_SCALE);
    }

    #[test]
    fn scale_just_below_threshold_proportional() {
        let h = SCALE_THRESHOLD_PX - 1.0;
        let s = compute_scale_factor(h);
        assert!(s < 1.0, "expected scale < 1.0, got {s}");
        assert!(s > MIN_SCALE, "expected scale > MIN_SCALE, got {s}");
    }
}
