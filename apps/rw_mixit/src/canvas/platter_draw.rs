/// Platter canvas drawing — vinyl disc, animated groove rotation, and tonearm.
///
/// `draw_platter` is called from the rAF loop once per frame.  It draws:
///   1. Vinyl disc background (filled dark circle).
///   2. Concentric groove rings, rotated by the accumulated playback angle.
///   3. Center label circle (not rotated) with the track name.
///   4. White spindle dot at the label center.
///   5. Tonearm line segment pivoting from the top-right, sweeping inward
///      from the outer groove to near the label over the full track duration.
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use leptos::html;
use leptos::prelude::*;
use crate::state::DeckState;
use std::f64::consts::TAU;

/// Intrinsic canvas size in pixels (square).  CSS may scale it for display.
pub const PLATTER_SIZE: u32 = 240;

// ── Drawing constants ─────────────────────────────────────────────────────────

/// Pixel gap between the canvas edge and the outer platter disc edge.
const PLATTER_PADDING: f64 = 8.0;

/// Center label circle radius as a fraction of the platter radius.
const LABEL_RADIUS_FRAC: f64 = 0.35;

/// Number of concentric vinyl groove rings drawn each frame.
const GROOVE_COUNT: u32 = 18;

/// Standard 33 RPM expressed as rotations per second (33 ÷ 60).
const RPM_33_RPS: f64 = 33.0 / 60.0;

// ── Tonearm geometry (for a 240 × 240 canvas, r ≈ 112 px) ────────────────────
//
// Derivation (all in canvas/screen coords where y increases downward):
//   pivot = (cx + r * 1.0, cy − r * 0.9) ≈ (232, 19)
//   outer groove at 3-o'clock: (cx + r * 0.90, cy) ≈ (221, 120)
//   Vector pivot→outer_tip: (−11, 101)
//   arm_length = sqrt(11² + 101²) ≈ 101.6   → fraction: 101.6 / 112 ≈ 0.907
//   start_angle = atan2(101, −11) ≈ 1.682 rad (≈ 96.3°, mostly downward)
//   sweep 45° CCW → tip lands at ≈ 44% of r (just outside label at 35%).

/// Tonearm pivot x-offset from platter center, in units of platter radius.
const TONEARM_PIVOT_DX: f64 = 1.0;

/// Tonearm pivot y-offset from platter center (negative = above center).
const TONEARM_PIVOT_DY: f64 = -0.9;

/// Tonearm arm length as a fraction of the platter radius.
const TONEARM_ARM_FRAC: f64 = 0.907;

/// Arm angle (radians from positive-x) when the needle is at the outer groove.
const TONEARM_START_ANGLE: f64 = 1.682;

/// Total arm sweep in radians over the full track duration (45°).
const TONEARM_MAX_SWEEP: f64 = std::f64::consts::FRAC_PI_4;

// ── Colours ───────────────────────────────────────────────────────────────────

const COLOR_DECK_A:  &str = "#3b82f6";
const COLOR_DECK_B:  &str = "#f97316";
const COLOR_VINYL:   &str = "#111111";
const COLOR_GROOVE:  &str = "#2a2a2a";
const COLOR_SPINDLE: &str = "#ffffff";
const COLOR_TONEARM: &str = "#cccccc";
const COLOR_BG:      &str = "#1a1a2e";

// ── Public entry point ────────────────────────────────────────────────────────

/// Draw the platter for one rAF frame.
///
/// Reads `DeckState` via `.get_untracked()` (no reactive tracking overhead).
/// `deck_side` is `"a"` or `"b"` and selects the label accent colour.
pub fn draw_platter(
    canvas_ref: &NodeRef<html::Canvas>,
    state:      &DeckState,
    deck_side:  &str,
) {
    let canvas = match canvas_ref.get_untracked() {
        Some(c) => c,
        None    => return,
    };
    let ctx = match canvas_2d(&canvas) {
        Some(c) => c,
        None    => return,
    };

    let w = canvas.width()  as f64;
    let h = canvas.height() as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let r  = (w.min(h) / 2.0) - PLATTER_PADDING;

    let current_secs  = state.current_secs.get_untracked();
    let duration_secs = state.duration_secs.get_untracked();
    let playback_rate = state.playback_rate.get_untracked();
    let track_name    = state.track_name.get_untracked();

    let accent = if deck_side == "a" { COLOR_DECK_A } else { COLOR_DECK_B };

    // ── 1. Clear canvas ───────────────────────────────────────────────────────
    ctx.set_fill_style_str(COLOR_BG);
    ctx.fill_rect(0.0, 0.0, w, h);

    // ── 2. Vinyl disc base (outer edge / label backing) ───────────────────────
    ctx.begin_path();
    ctx.arc(cx, cy, r, 0.0, TAU).expect("platter: disc arc");
    ctx.set_fill_style_str(COLOR_VINYL);
    ctx.fill();

    // ── 3. Groove rings — rotated with accumulated playback angle ─────────────
    //
    // angle = elapsed_revolutions × 2π
    // elapsed_revolutions = current_secs × RPM_33_RPS × playback_rate
    let angle = current_secs * RPM_33_RPS * playback_rate * TAU;

    ctx.save();
    ctx.translate(cx, cy).expect("platter: translate");
    ctx.rotate(angle).expect("platter: rotate");

    ctx.set_stroke_style_str(COLOR_GROOVE);
    ctx.set_line_width(0.8);

    let outer_groove_r = r * 0.92;
    let inner_groove_r = r * 0.38; // just outside the label circle
    let groove_spacing = (outer_groove_r - inner_groove_r) / GROOVE_COUNT as f64;

    for i in 0..GROOVE_COUNT {
        let groove_r = inner_groove_r + i as f64 * groove_spacing;
        ctx.begin_path();
        ctx.arc(0.0, 0.0, groove_r, 0.0, TAU).expect("platter: groove arc");
        ctx.stroke();
    }

    ctx.restore();

    // ── 4. Center label circle (static — not part of the rotation) ───────────
    let label_r = r * LABEL_RADIUS_FRAC;

    ctx.begin_path();
    ctx.arc(cx, cy, label_r, 0.0, TAU).expect("platter: label arc");
    ctx.set_fill_style_str(accent);
    ctx.fill();

    // Label outline
    ctx.set_stroke_style_str(COLOR_VINYL);
    ctx.set_line_width(2.0);
    ctx.stroke();

    // Track name on label (abbreviated, Bangers font)
    if let Some(ref name) = track_name {
        let abbrev = truncate_label(name, 10);
        ctx.set_fill_style_str(COLOR_SPINDLE);
        ctx.set_font("bold 10px Bangers, cursive");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text(&abbrev, cx, cy - label_r * 0.25)
            .expect("platter: fill_text");
    }

    // ── 5. Spindle dot ────────────────────────────────────────────────────────
    const SPINDLE_R: f64 = 4.0;
    ctx.begin_path();
    ctx.arc(cx, cy + label_r * 0.3, SPINDLE_R, 0.0, TAU)
        .expect("platter: spindle arc");
    ctx.set_fill_style_str(COLOR_SPINDLE);
    ctx.fill();

    // ── 6. Tonearm ────────────────────────────────────────────────────────────
    let pivot_x = cx + r * TONEARM_PIVOT_DX;
    let pivot_y = cy + r * TONEARM_PIVOT_DY;
    let arm_len = r * TONEARM_ARM_FRAC;

    let progress = if duration_secs > 0.0 {
        (current_secs / duration_secs).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let arm_angle = TONEARM_START_ANGLE + progress * TONEARM_MAX_SWEEP;

    let tip_x = pivot_x + arm_len * arm_angle.cos();
    let tip_y = pivot_y + arm_len * arm_angle.sin();

    // Arm line
    ctx.set_stroke_style_str(COLOR_TONEARM);
    ctx.set_line_width(3.5);
    ctx.set_line_cap("round");
    ctx.begin_path();
    ctx.move_to(pivot_x, pivot_y);
    ctx.line_to(tip_x, tip_y);
    ctx.stroke();

    // Pivot disc
    ctx.begin_path();
    ctx.arc(pivot_x, pivot_y, 5.0, 0.0, TAU).expect("platter: pivot arc");
    ctx.set_fill_style_str(COLOR_TONEARM);
    ctx.fill();

    // Needle cartridge dot (accent colour)
    ctx.begin_path();
    ctx.arc(tip_x, tip_y, 3.0, 0.0, TAU).expect("platter: needle arc");
    ctx.set_fill_style_str(accent);
    ctx.fill();
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Get the 2D rendering context from a canvas element.
fn canvas_2d(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}

/// Abbreviate a track name to fit on the label.
///
/// Strips the file extension, then truncates with "…" if longer than `max_len`.
pub fn truncate_label(name: &str, max_len: usize) -> String {
    let stem = match name.rfind('.') {
        Some(i) => &name[..i],
        None    => name,
    };
    if stem.len() <= max_len {
        stem.to_string()
    } else {
        format!("{}…", &stem[..max_len - 1])
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_label_strips_extension() {
        assert_eq!(truncate_label("boombap_vol1.mp3", 20), "boombap_vol1");
    }

    #[test]
    fn truncate_label_no_extension() {
        assert_eq!(truncate_label("hello", 12), "hello");
    }

    #[test]
    fn truncate_label_long_gets_ellipsis() {
        let result = truncate_label("averylongtracknamewithoutext", 10);
        assert!(result.ends_with('…'), "expected ellipsis, got: {result}");
        assert_eq!(result.chars().count(), 10);
    }

    #[test]
    fn truncate_label_long_with_extension() {
        let result = truncate_label("averylongtracknamewithoutext.flac", 10);
        assert!(result.ends_with('…'));
    }

    #[test]
    fn tonearm_at_start() {
        let angle = TONEARM_START_ANGLE + 0.0 * TONEARM_MAX_SWEEP;
        assert!((angle - TONEARM_START_ANGLE).abs() < 1e-12);
    }

    #[test]
    fn tonearm_at_end() {
        let angle = TONEARM_START_ANGLE + 1.0 * TONEARM_MAX_SWEEP;
        assert!((angle - (TONEARM_START_ANGLE + TONEARM_MAX_SWEEP)).abs() < 1e-12);
    }

    #[test]
    fn groove_rotation_zero_at_rest() {
        // When current_secs == 0, rotation angle is 0.
        let angle = 0.0_f64 * RPM_33_RPS * 1.0 * TAU;
        assert!(angle.abs() < 1e-12);
    }

    #[test]
    fn groove_rotation_one_full_revolution() {
        // After 1 / RPM_33_RPS seconds at playback_rate 1.0, exactly one full rotation.
        let secs_per_rev = 1.0 / RPM_33_RPS;
        let angle = secs_per_rev * RPM_33_RPS * 1.0 * TAU;
        assert!((angle - TAU).abs() < 1e-9, "expected TAU, got {angle}");
    }

    // ── WASM smoke tests (require browser DOM) ────────────────────────────────
    #[cfg(target_arch = "wasm32")]
    mod wasm {
        use super::*;
        use leptos::prelude::*;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_test::wasm_bindgen_test;
        use crate::state::DeckState;

        fn make_canvas() -> web_sys::HtmlCanvasElement {
            let doc = web_sys::window().unwrap().document().unwrap();
            let el = doc.create_element("canvas").unwrap();
            el.set_attribute("width", "240").unwrap();
            el.set_attribute("height", "240").unwrap();
            el.unchecked_into()
        }

        /// Empty NodeRef (no canvas mounted yet) must return early, not panic.
        #[wasm_bindgen_test]
        fn draw_platter_empty_ref_does_not_panic() {
            let owner = Owner::new();
            owner.with(|| {
                let state = DeckState::new();
                let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
                draw_platter(&canvas_ref, &state, "a");
            });
        }

        /// Real canvas element — both deck sides must draw without panicking.
        #[wasm_bindgen_test]
        fn draw_platter_with_canvas_does_not_panic() {
            let owner = Owner::new();
            owner.with(|| {
                let state = DeckState::new();
                state.duration_secs.set(180.0);
                state.current_secs.set(30.0);

                let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
                canvas_ref.load(&make_canvas());

                draw_platter(&canvas_ref, &state, "a");
                draw_platter(&canvas_ref, &state, "b");
            });
        }

        /// Tonearm at end of track (progress = 1.0) must not panic.
        #[wasm_bindgen_test]
        fn draw_platter_at_track_end_does_not_panic() {
            let owner = Owner::new();
            owner.with(|| {
                let state = DeckState::new();
                state.duration_secs.set(180.0);
                state.current_secs.set(180.0);

                let canvas_ref = NodeRef::<leptos::html::Canvas>::new();
                canvas_ref.load(&make_canvas());

                draw_platter(&canvas_ref, &state, "a");
            });
        }
    }
}
