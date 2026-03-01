use web_sys::CanvasRenderingContext2d;
use crate::graphics::colors::color_to_css;
use crate::graphics::RetroColors;

/// Thin abstraction over a Canvas 2D context for game rendering.
pub struct Renderer {
    pub ctx: CanvasRenderingContext2d,
    pub width: f64,
    pub height: f64,
}

impl Renderer {
    pub fn new(ctx: CanvasRenderingContext2d, width: f64, height: f64) -> Self {
        // Disable image smoothing for crisp pixel art
        ctx.set_image_smoothing_enabled(false);
        Renderer { ctx, width, height }
    }

    pub fn clear(&self) {
        self.ctx.set_fill_style_str("rgb(0,0,8)");
        self.ctx.fill_rect(0.0, 0.0, self.width, self.height);
    }

    pub fn fill_rect(&self, x: f64, y: f64, w: f64, h: f64, color: u32) {
        let css = color_to_css(color);
        self.ctx.set_fill_style_str(&css);
        self.ctx.fill_rect(x, y, w, h);
    }

    pub fn draw_text(&self, text: &str, x: f64, y: f64, color: u32, size: u32) {
        let css = color_to_css(color);
        self.ctx.set_fill_style_str(&css);
        self.ctx.set_font(&format!("{}px monospace", size));
        let _ = self.ctx.fill_text(text, x, y);
    }

    pub fn draw_text_centered(&self, text: &str, y: f64, color: u32, size: u32) {
        // Approximate centering: each monospace char is ~0.6 * size wide
        let approx_width = text.len() as f64 * size as f64 * 0.6;
        let x = (self.width - approx_width) / 2.0;
        self.draw_text(text, x, y, color, size);
    }

    /// Draw a horizontal bar (e.g. for power-up timer).
    pub fn draw_bar(&self, x: f64, y: f64, width: f64, height: f64, fraction: f64, color: u32) {
        // Background
        self.fill_rect(x, y, width, height, RetroColors::DARK_GRAY);
        // Filled portion
        let filled = (width * fraction.clamp(0.0, 1.0)).max(0.0);
        if filled > 0.0 {
            self.fill_rect(x, y, filled, height, color);
        }
    }

    /// Draw a simple rectangle outline.
    #[allow(dead_code)]
    pub fn draw_rect_outline(&self, x: f64, y: f64, w: f64, h: f64, color: u32) {
        let css = color_to_css(color);
        self.ctx.set_stroke_style_str(&css);
        self.ctx.stroke_rect(x, y, w, h);
    }

    pub fn set_alpha(&self, alpha: f64) {
        self.ctx.set_global_alpha(alpha);
    }

    pub fn reset_alpha(&self) {
        self.ctx.set_global_alpha(1.0);
    }
}
