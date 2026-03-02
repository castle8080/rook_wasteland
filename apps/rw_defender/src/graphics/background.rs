/// Parallax starfield + NASA/ESA background image rendered to the background canvas.
///
/// Three depth layers scroll at different speeds to create depth illusion.
/// Star colors shift based on the current wave tier. The real space photo is
/// drawn first on each frame (via drawImage) and the star particles are
/// composited on top.
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

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

/// Background image filenames served from /backgrounds/ (Trunk copies assets/backgrounds/).
/// Each tier has a slice of images cycled by wave number within that tier.
const WARM_IMAGES: &[&str] = &[
    "esa_orion_nebula_m42.jpg",
    "esa_mystic_mountain_carina.jpg",
    "esa_eagle_nebula_m16.jpg",
    "esa_carina_nebula_hubble.jpg",
    "esa_helix_nebula.jpg",
];

const NEBULA_IMAGES: &[&str] = &[
    "esa_butterfly_nebula.jpg",
    "esa_crab_nebula.jpg",
    "esa_ring_nebula_m57.jpg",
    "08_hubble_ring_nebula_m57.jpg",
    "02_jwst_southern_ring_nebula.jpg",
    "03_jwst_hidden_orion.jpg",
];

const DEEP_IMAGES: &[&str] = &[
    "esa_sombrero_galaxy.jpg",
    "esa_andromeda_galaxy_m31.jpg",
    "esa_whirlpool_galaxy_m51.jpg",
    "esa_antennae_galaxies.jpg",
    "esa_ngc1300_barred_spiral.jpg",
    "05_vlt_ngc1232_spiral_galaxy.jpg",
];

const ULTRA_DEEP_IMAGES: &[&str] = &[
    "01_jwst_deep_field_smacs0723.jpg",
    "04_hubble_cats_eye_nebula.jpg",
    "esa_ngc3603_stellar_nursery.jpg",
    "esa_hoags_object_ring_galaxy.jpg",
    "esa_omega_centauri_cluster.jpg",
    "18_hubble_omega_nebula_m17.jpg",
];

/// Returns the URL path for the background image to use for the given wave.
/// Wave 1 → first Warm image, wave 2 → second Warm image, etc. Cycles within tier.
pub fn background_image_for_wave(wave: u32) -> &'static str {
    match BackgroundTier::for_wave(wave) {
        BackgroundTier::Warm => {
            let idx = ((wave - 1) as usize) % WARM_IMAGES.len();
            WARM_IMAGES[idx]
        }
        BackgroundTier::Nebula => {
            let idx = ((wave - 6) as usize) % NEBULA_IMAGES.len();
            NEBULA_IMAGES[idx]
        }
        BackgroundTier::Deep => {
            let idx = ((wave - 11) as usize) % DEEP_IMAGES.len();
            DEEP_IMAGES[idx]
        }
        BackgroundTier::UltraDeep => {
            let idx = ((wave.saturating_sub(16)) as usize) % ULTRA_DEEP_IMAGES.len();
            ULTRA_DEEP_IMAGES[idx]
        }
    }
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
    /// Current space photo, loaded asynchronously via HtmlImageElement.
    bg_image: Option<HtmlImageElement>,
    /// Filename last set (to avoid recreating the element on every wave check).
    bg_filename: String,
}

impl StarField {
    pub fn new(canvas: &HtmlCanvasElement) -> Self {
        let ctx = canvas
            .get_context("2d")
            .expect("bg get_context failed")
            .expect("no bg 2d context")
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("bg context not CanvasRenderingContext2d");

        ctx.set_image_smoothing_enabled(true);

        let layers = [
            Self::generate_layer(0, 80),  // slow, small
            Self::generate_layer(1, 50),  // medium
            Self::generate_layer(2, 25),  // fast, large, bright
        ];

        StarField { ctx, layers, tier: BackgroundTier::Warm, bg_image: None, bg_filename: String::new() }
    }

    /// Set the background image for the current wave. `filename` is the base name
    /// (e.g. `"esa_orion_nebula_m42.jpg"`). The image is loaded at `/backgrounds/<filename>`.
    pub fn set_background_image(&mut self, filename: &str) {
        if filename == self.bg_filename {
            return; // already loading/loaded this one
        }
        self.bg_filename = filename.to_string();
        let img = HtmlImageElement::new().expect("HtmlImageElement::new failed");
        img.set_src(&format!("/backgrounds/{filename}"));
        self.bg_image = Some(img);
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

        // Clear canvas to transparent so background shows if image not ready yet.
        ctx.clear_rect(0.0, 0.0, CANVAS_W, CANVAS_H);

        // Draw the NASA/ESA background image if loaded (img.complete() is true once decoded).
        if let Some(ref img) = self.bg_image {
            if img.complete() && img.natural_width() > 0 {
                // Draw at 45% opacity to keep image visible but not overwhelming.
                ctx.set_global_alpha(0.45);
                let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    img, 0.0, 0.0, CANVAS_W, CANVAS_H,
                );
                ctx.set_global_alpha(1.0);
            }
        }

        // Subtle tier-colored nebula tint (transparent overlay, not a solid fill).
        if let Some((color, _alpha)) = self.tier.nebula_color() {
            ctx.set_fill_style_str(color);
            ctx.fill_rect(0.0, 0.0, CANVAS_W, CANVAS_H);
        }

        // Draw stars per layer.
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

    #[test]
    fn test_background_image_for_wave_returns_jpg() {
        // Every wave 0–30 must return a .jpg filename
        for wave in 0u32..=30 {
            let f = background_image_for_wave(wave);
            assert!(f.ends_with(".jpg"), "wave {wave}: expected .jpg, got {f}");
        }
    }

    #[test]
    fn test_background_image_for_wave_known_values() {
        // Wave 0 → UltraDeep (default branch), first image
        assert_eq!(background_image_for_wave(0), "01_jwst_deep_field_smacs0723.jpg");
        // Wave 1 → first Warm image
        assert_eq!(background_image_for_wave(1), "esa_orion_nebula_m42.jpg");
        // Wave 5 → last Warm image (5-1=4 mod 5 = 4)
        assert_eq!(background_image_for_wave(5), "esa_helix_nebula.jpg");
        // Wave 6 → first Nebula image
        assert_eq!(background_image_for_wave(6), "esa_butterfly_nebula.jpg");
        // Wave 11 → first Deep image
        assert_eq!(background_image_for_wave(11), "esa_sombrero_galaxy.jpg");
        // Wave 16 → first UltraDeep image
        assert_eq!(background_image_for_wave(16), "01_jwst_deep_field_smacs0723.jpg");
    }

    #[test]
    fn test_background_image_cycles_within_tier() {
        // Warm: waves 1-5 each return different images, wave 6 goes to Nebula
        let warm_images: Vec<_> = (1u32..=5).map(background_image_for_wave).collect();
        // All 5 warm images should be distinct
        let unique: std::collections::HashSet<_> = warm_images.iter().collect();
        assert_eq!(unique.len(), 5, "expected 5 distinct warm images");
        // Wave 6 must NOT be same as warm images
        assert!(!warm_images.contains(&background_image_for_wave(6)));
    }

    #[test]
    fn test_ultra_deep_large_wave() {
        // Very large wave numbers should not panic (cycle wraps)
        let f = background_image_for_wave(u32::MAX);
        assert!(f.ends_with(".jpg"));
    }
}
