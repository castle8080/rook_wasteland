/// Waveform canvas drawing — static peak render + dynamic playhead overlay.
///
/// Two-pass approach per rAF frame:
/// 1. **Static pass** (`draw_waveform_static`): renders `waveform_peaks` into a
///    detached `HtmlCanvasElement` that is kept in an `Rc<RefCell<Option<…>>>`.
///    Redrawn only when the peaks data changes (detected by comparing the stored
///    peak count against the previous frame's count).
/// 2. **Dynamic pass** (`draw_waveform_dynamic`): composites the offscreen canvas
///    onto the visible canvas and draws the moving playhead, loop region, and hot
///    cue markers.
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d};
use leptos::html;
use leptos::prelude::*;
use crate::state::DeckState;

/// Deck-accent hex colors for waveform bars; A = blue, B = orange.
const COLOR_DECK_A: &str = "#3b82f6";
const COLOR_DECK_B: &str = "#f97316";
/// Playhead line color.
const COLOR_PLAYHEAD: &str = "#ffffff";
/// Loop region highlight color (translucent).
const COLOR_LOOP: &str = "rgba(255, 230, 0, 0.25)";
/// Background color inside the waveform canvas.
const COLOR_BG: &str = "#1a1a2e";

/// Cached offscreen canvas state for one deck.
pub struct WaveformCache {
    /// Detached canvas holding the static peak render.
    canvas:     Option<HtmlCanvasElement>,
    /// Number of peaks in the last drawn snapshot (used to detect changes).
    peak_count: usize,
    /// Zoom level at the time of the last static draw.
    zoom:       u8,
}

impl WaveformCache {
    pub fn new() -> Rc<RefCell<WaveformCache>> {
        Rc::new(RefCell::new(WaveformCache {
            canvas: None,
            peak_count: 0,
            zoom: 0, // 0 = "never drawn" sentinel
        }))
    }
}

// ── Public draw entry point ───────────────────────────────────────────────────

/// Full waveform draw: static pass (if peaks changed) then dynamic overlay.
///
/// `deck_side` is `"a"` or `"b"` to pick the accent color.
pub fn draw_waveform(
    canvas_ref: &NodeRef<html::Canvas>,
    state:      &DeckState,
    cache:      &Rc<RefCell<WaveformCache>>,
    deck_side:  &str,
) {
    let canvas = match canvas_ref.get_untracked() {
        Some(c) => c,
        None => return,
    };
    let ctx2d = match canvas_2d(&canvas) {
        Some(c) => c,
        None => return,
    };

    let width  = canvas.width()  as f64;
    let height = canvas.height() as f64;
    let peaks_opt = state.waveform_peaks.get_untracked();
    let zoom      = state.zoom_level.get_untracked();
    let current   = state.current_secs.get_untracked();
    let duration  = state.duration_secs.get_untracked();

    // ── Step 1: redraw static offscreen canvas if needed ─────────────────────
    {
        let mut c = cache.borrow_mut();
        let needs_redraw = match &peaks_opt {
            Some(p) => c.peak_count != p.len() || c.zoom != zoom,
            None    => false,
        };
        if needs_redraw {
            if let Some(ref peaks) = peaks_opt {
                let offscreen = make_offscreen_canvas(width as u32, height as u32);
                let color = if deck_side == "a" { COLOR_DECK_A } else { COLOR_DECK_B };
                draw_peaks_to_canvas(&offscreen, peaks, zoom, height, color);
                c.canvas     = Some(offscreen);
                c.peak_count = peaks.len();
                c.zoom       = zoom;
            }
        }
    }

    // ── Step 2: clear the visible canvas ─────────────────────────────────────
    ctx2d.set_fill_style_str(COLOR_BG);
    ctx2d.fill_rect(0.0, 0.0, width, height);

    // ── Step 3: composite the static waveform ────────────────────────────────
    // Scroll so the playhead is always centered.
    let center_x = width / 2.0;
    let progress = if duration > 0.0 { current / duration } else { 0.0 };
    let total_peak_width = width; // static canvas is same size as display canvas
    let scroll_x = -(progress * total_peak_width - center_x);

    {
        let c = cache.borrow();
        if let Some(ref offscreen) = c.canvas {
            ctx2d.draw_image_with_html_canvas_element(offscreen, scroll_x, 0.0)
                .expect("draw_waveform — drawImage");
        }
    }

    // Draw waveform peeking in from the other side (seamless scroll wrap).
    {
        let c = cache.borrow();
        if let Some(ref offscreen) = c.canvas {
            if scroll_x > 0.0 {
                // Beginning of track scrolled right — fill left gap with dark.
            } else if scroll_x + total_peak_width < width {
                // End of track — nothing to wrap.
                let _ = ctx2d.draw_image_with_html_canvas_element(
                    offscreen,
                    scroll_x + total_peak_width,
                    0.0,
                );
            }
        }
    }

    // ── Step 4: loop region overlay ───────────────────────────────────────────
    if state.loop_active.get_untracked() && duration > 0.0 {
        let loop_in  = state.loop_in.get_untracked();
        let loop_out = state.loop_out.get_untracked();
        if loop_out > loop_in {
            let x_in  = time_to_x(loop_in,  duration, scroll_x, total_peak_width);
            let x_out = time_to_x(loop_out, duration, scroll_x, total_peak_width);
            ctx2d.set_fill_style_str(COLOR_LOOP);
            ctx2d.fill_rect(x_in, 0.0, x_out - x_in, height);
        }
    }

    // ── Step 5: playhead line ─────────────────────────────────────────────────
    ctx2d.set_stroke_style_str(COLOR_PLAYHEAD);
    ctx2d.set_line_width(2.0);
    ctx2d.begin_path();
    ctx2d.move_to(center_x, 0.0);
    ctx2d.line_to(center_x, height);
    ctx2d.stroke();
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Render peak bars for the given zoom level into `offscreen`.
fn draw_peaks_to_canvas(
    offscreen: &HtmlCanvasElement,
    peaks:     &[f32],
    zoom:       u8,
    height:     f64,
    color:      &str,
) {
    let ctx2d = match canvas_2d(offscreen) {
        Some(c) => c,
        None => return,
    };
    let width = offscreen.width() as f64;

    // Clear.
    ctx2d.set_fill_style_str(COLOR_BG);
    ctx2d.fill_rect(0.0, 0.0, width, height);

    if peaks.is_empty() {
        return;
    }

    // Determine the slice of peaks visible at this zoom level.
    let n_total  = peaks.len();
    let n_visible = (n_total / zoom.max(1) as usize).max(1);
    let start_col = 0usize; // at zoom > 1 the start shifts with playhead — handled in dynamic pass
    let end_col   = n_visible.min(n_total);
    let visible   = &peaks[start_col..end_col];

    let bar_w = width / visible.len() as f64;
    let mid_y = height / 2.0;

    ctx2d.set_fill_style_str(color);
    for (i, &peak) in visible.iter().enumerate() {
        let x        = i as f64 * bar_w;
        let bar_h    = (peak as f64 * mid_y).max(1.0);
        ctx2d.fill_rect(x, mid_y - bar_h, bar_w.max(1.0), bar_h * 2.0);
    }
}

/// Compute x-pixel position of a time value on the scrolled waveform.
#[inline]
fn time_to_x(time: f64, duration: f64, scroll_x: f64, total_width: f64) -> f64 {
    (time / duration) * total_width + scroll_x
}

/// Get the 2D rendering context from a canvas element.
fn canvas_2d(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}

/// Create a detached (not in the DOM) canvas of the given dimensions.
fn make_offscreen_canvas(width: u32, height: u32) -> HtmlCanvasElement {
    let window   = web_sys::window().expect("no window");
    let document = window.document().expect("no document");
    let canvas   = document
        .create_element("canvas")
        .expect("create_element canvas")
        .dyn_into::<HtmlCanvasElement>()
        .expect("dyn_into HtmlCanvasElement");
    canvas.set_width(width);
    canvas.set_height(height);
    canvas
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::time_to_x;

    #[test]
    fn time_to_x_at_start_equals_scroll_x() {
        // time=0 should map to scroll_x (the left edge of the waveform).
        let x = time_to_x(0.0, 180.0, -200.0, 800.0);
        assert!((x - (-200.0)).abs() < 1e-9);
    }

    #[test]
    fn time_to_x_at_end() {
        let x = time_to_x(180.0, 180.0, 0.0, 800.0);
        assert!((x - 800.0).abs() < 1e-9);
    }

    #[test]
    fn time_to_x_midpoint() {
        let x = time_to_x(90.0, 180.0, 0.0, 800.0);
        assert!((x - 400.0).abs() < 1e-9);
    }
}
