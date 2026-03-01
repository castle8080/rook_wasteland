/// Procedural parallax starfield rendered to the background canvas.
///
/// Three depth layers scroll at different speeds to create depth illusion.
/// Star colors shift based on the current wave tier.
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::utils::math::lcg_rand;

const CANVAS_W: f64 = 640.0;
const CANVAS_H: f64 = 480.0;

/// One star particle.
#[derive(Clone)]
struct Star {
    x: f64,
    y: f64,
    brightness: f64, // 0.0–1.0
    scroll_speed: f64,
}

/// Which tier of background (changes color temperature).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundTier {
    /// Waves 1–5: warm reddish Mars-like
    Warm,
    /// Waves 6–10: blue-white nebula
    Nebula,
    /// Waves 11–15: purple deep space
    Deep,
    /// Waves 16+: cold blue ultra-deep field
    UltraDeep,
}

impl BackgroundTier {
    pub fn for_wave(wave: u32) -> Self {
        match wave {
            1..=5 => BackgroundTier::Warm,
            6..=10 => BackgroundTier::Nebula,
            11..=15 => BackgroundTier::Deep,
            _ => BackgroundTier::UltraDeep,
        }
    }

    fn star_color(&self, brightness: f64) -> String {
        let b = (brightness * 255.0) as u8;
        match self {
            BackgroundTier::Warm => {
                // Warm white with slight red tint
                format!("rgba({},{},{},{})", b, (b as f64 * 0.85) as u8, (b as f64 * 0.70) as u8, brightness)
            }
            BackgroundTier::Nebula => {
                // Cool blue-white
                format!("rgba({},{},{},{})", (b as f64 * 0.70) as u8, (b as f64 * 0.85) as u8, b, brightness)
            }
            BackgroundTier::Deep => {
                // Purple tint
                format!("rgba({},{},{},{})", (b as f64 * 0.85) as u8, (b as f64 * 0.65) as u8, b, brightness)
            }
            BackgroundTier::UltraDeep => {
                // Icy blue-white
                format!("rgba({},{},{},{})", (b as f64 * 0.60) as u8, (b as f64 * 0.80) as u8, b, brightness)
            }
        }
    }

    fn nebula_color(&self) -> Option<(&'static str, f64)> {
        match self {
            BackgroundTier::Nebula => Some(("rgba(30, 60, 120, 0.12)", 0.12)),
            BackgroundTier::Deep => Some(("rgba(60, 20, 80, 0.15)", 0.15)),
            BackgroundTier::UltraDeep => Some(("rgba(10, 20, 60, 0.18)", 0.18)),
            BackgroundTier::Warm => None,
        }
    }
}

pub struct StarField {
    ctx: CanvasRenderingContext2d,
    /// Layer 0: slow/small/dim; Layer 1: medium; Layer 2: fast/large/bright
    layers: [Vec<Star>; 3],
    pub tier: BackgroundTier,
}

impl StarField {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let ctx = canvas
            .get_context("2d")
            .expect("bg get_context failed")
            .expect("no bg 2d context")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("bg context not CanvasRenderingContext2d");

        ctx.set_image_smoothing_enabled(false);

        let layers = [
            Self::generate_layer(0, 80),  // slow, small
            Self::generate_layer(1, 50),  // medium
            Self::generate_layer(2, 25),  // fast, large, bright
        ];

        StarField { ctx, layers, tier: BackgroundTier::Warm }
    }

    fn generate_layer(layer: usize, count: usize) -> Vec<Star> {
        let mut seed: u64 = 0xDEAD_BEEF ^ (layer as u64 * 0x9E37_79B9);
        let (speed, brightness_base) = match layer {
            0 => (15.0, 0.4),
            1 => (35.0, 0.65),
            _ => (65.0, 0.9),
        };

        (0..count)
            .map(|_| {
                let rx = lcg_rand(&mut seed);
                let ry = lcg_rand(&mut seed);
                let rb = lcg_rand(&mut seed);
                Star {
                    x: rx * CANVAS_W,
                    y: ry * CANVAS_H,
                    brightness: brightness_base * (0.5 + rb * 0.5),
                    scroll_speed: speed * (0.8 + rb * 0.4),
                }
            })
            .collect()
    }

    pub fn set_tier(&mut self, tier: BackgroundTier) {
        self.tier = tier;
    }

    pub fn update(&mut self, dt: f64) {
        for layer in &mut self.layers {
            for star in layer.iter_mut() {
                star.y += star.scroll_speed * dt;
                if star.y > CANVAS_H {
                    star.y -= CANVAS_H;
                }
            }
        }
    }

    pub fn render(&self) {
        let ctx = &self.ctx;

        // Black background
        ctx.set_fill_style_str("#000005");
        ctx.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H);

        // Optional nebula glow (radial)
        if let Some((color, _alpha)) = self.tier.nebula_color() {
            ctx.set_fill_style_str(color);
            ctx.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H);
        }

        // Draw stars per layer
        for (li, layer) in self.layers.iter().enumerate() {
            let size = match li {
                0 => 1.0,
                1 => 1.5,
                _ => 2.5,
            };
            for star in layer {
                let color = self.tier.star_color(star.brightness);
                ctx.set_fill_style_str(&color);
                ctx.fill_rect(star.x, star.y, size, size);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_tier_for_wave() {
        assert_eq!(BackgroundTier::for_wave(1), BackgroundTier::Warm);
        assert_eq!(BackgroundTier::for_wave(5), BackgroundTier::Warm);
        assert_eq!(BackgroundTier::for_wave(6), BackgroundTier::Nebula);
        assert_eq!(BackgroundTier::for_wave(10), BackgroundTier::Nebula);
        assert_eq!(BackgroundTier::for_wave(11), BackgroundTier::Deep);
        assert_eq!(BackgroundTier::for_wave(15), BackgroundTier::Deep);
        assert_eq!(BackgroundTier::for_wave(16), BackgroundTier::UltraDeep);
        assert_eq!(BackgroundTier::for_wave(50), BackgroundTier::UltraDeep);
    }

    #[test]
    fn test_star_layer_counts() {
        let l0 = StarField::generate_layer(0, 80);
        let l2 = StarField::generate_layer(2, 25);
        assert_eq!(l0.len(), 80);
        assert_eq!(l2.len(), 25);
    }

    #[test]
    fn test_star_scrolling_wraps() {
        let mut layer = StarField::generate_layer(0, 10);
        // Force all stars to near bottom
        for s in &mut layer {
            s.y = CANVAS_H - 1.0;
        }
        // Manually simulate update
        for s in &mut layer {
            s.y += s.scroll_speed * 1.0; // 1 second
            if s.y > CANVAS_H {
                s.y -= CANVAS_H;
            }
        }
        // All should have wrapped (scroll_speed > 1.0 for layer 0)
        assert!(layer.iter().all(|s| s.y < CANVAS_H));
    }

    #[test]
    fn test_tier_colors_differ() {
        let warm = BackgroundTier::Warm.star_color(0.8);
        let nebula = BackgroundTier::Nebula.star_color(0.8);
        let deep = BackgroundTier::Deep.star_color(0.8);
        assert_ne!(warm, nebula);
        assert_ne!(warm, deep);
        assert_ne!(nebula, deep);
    }
}
