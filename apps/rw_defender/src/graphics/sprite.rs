use web_sys::CanvasRenderingContext2d;
use super::colors::{RetroColors, color_to_css};

/// A static pixel-art sprite stored as packed 0xAARRGGBB pixels.
#[derive(Clone)]
pub struct Sprite {
    pub pixels: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl Sprite {
    pub fn new(pixels: Vec<u32>, width: u32, height: u32) -> Self {
        Sprite { pixels, width, height }
    }

    /// Draws the sprite with batching of same-colored pixels per row.
    /// Transparent pixels (alpha < 10) are skipped.
    pub fn draw(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
        let mut last_color: u32 = 0;
        let mut has_color = false;

        for py in 0..self.height {
            for px in 0..self.width {
                let pixel = self.pixels[(py * self.width + px) as usize];
                let alpha = (pixel >> 24) & 0xFF;
                if alpha < 10 {
                    continue;
                }
                if !has_color || pixel != last_color {
                    let css = color_to_css(pixel);
                    ctx.set_fill_style_str(&css);
                    last_color = pixel;
                    has_color = true;
                }
                ctx.fill_rect(x + px as f64, y + py as f64, 1.0, 1.0);
            }
        }
    }
}

/// A looping or one-shot animated sprite.
#[allow(dead_code)]
pub struct AnimatedSprite {
    pub frames: Vec<Sprite>,
    pub frame_duration: f64,
    pub current_frame: usize,
    pub elapsed_time: f64,
    pub looping: bool,
    pub finished: bool,
}

#[allow(dead_code)]
impl AnimatedSprite {
    pub fn new(frames: Vec<Sprite>, frame_duration: f64, looping: bool) -> Self {
        AnimatedSprite {
            frames,
            frame_duration,
            current_frame: 0,
            elapsed_time: 0.0,
            looping,
            finished: false,
        }
    }

    pub fn update(&mut self, dt: f64) {
        if self.finished {
            return;
        }
        self.elapsed_time += dt;
        while self.elapsed_time >= self.frame_duration {
            self.elapsed_time -= self.frame_duration;
            self.current_frame += 1;
            if self.current_frame >= self.frames.len() {
                if self.looping {
                    self.current_frame = 0;
                } else {
                    self.current_frame = self.frames.len() - 1;
                    self.finished = true;
                    return;
                }
            }
        }
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
        if let Some(frame) = self.frames.get(self.current_frame) {
            frame.draw(ctx, x, y);
        }
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.elapsed_time = 0.0;
        self.finished = false;
    }
}

/// Procedural sprite generator for all game entities.
pub struct SpriteGenerator;

impl SpriteGenerator {
    // ── Player ───────────────────────────────────────────────────────────────

    pub fn player_ship() -> Sprite {
        // 16x16 player ship
        #[rustfmt::skip]
        let pattern: [u8; 256] = [
            0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,
            0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,0,
            0,0,0,0,0,0,1,2,2,1,0,0,0,0,0,0,
            0,0,0,0,0,1,2,2,2,2,1,0,0,0,0,0,
            0,0,0,0,1,2,2,3,3,2,2,1,0,0,0,0,
            0,0,0,1,2,2,3,3,3,3,2,2,1,0,0,0,
            0,0,1,2,2,2,3,3,3,3,2,2,2,1,0,0,
            0,1,2,2,2,2,2,3,3,2,2,2,2,2,1,0,
            1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,1,
            1,1,2,2,2,2,2,2,2,2,2,2,2,2,1,1,
            0,1,1,2,2,2,2,2,2,2,2,2,2,1,1,0,
            0,0,1,1,2,2,2,2,2,2,2,2,1,1,0,0,
            0,0,0,1,1,2,2,2,2,2,2,1,1,0,0,0,
            0,0,0,0,1,1,1,0,0,1,1,1,0,0,0,0,
            0,0,0,1,1,0,0,0,0,0,0,1,1,0,0,0,
            0,0,1,1,0,0,0,0,0,0,0,0,1,1,0,0,
        ];
        let palette = [
            RetroColors::TRANSPARENT,
            RetroColors::DARK_GRAY,
            RetroColors::BLUE,
            RetroColors::CYAN,
        ];
        Sprite::new(pattern.iter().map(|&i| palette[i as usize]).collect(), 16, 16)
    }

    pub fn player_ship_thrust() -> Sprite {
        // Variant with engine glow (bottom row shows thrust)
        #[rustfmt::skip]
        let pattern: [u8; 256] = [
            0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,
            0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,0,
            0,0,0,0,0,0,1,2,2,1,0,0,0,0,0,0,
            0,0,0,0,0,1,2,2,2,2,1,0,0,0,0,0,
            0,0,0,0,1,2,2,3,3,2,2,1,0,0,0,0,
            0,0,0,1,2,2,3,3,3,3,2,2,1,0,0,0,
            0,0,1,2,2,2,3,3,3,3,2,2,2,1,0,0,
            0,1,2,2,2,2,2,3,3,2,2,2,2,2,1,0,
            1,2,2,2,2,2,2,2,2,2,2,2,2,2,2,1,
            1,1,2,2,2,2,2,2,2,2,2,2,2,2,1,1,
            0,1,1,2,2,2,2,2,2,2,2,2,2,1,1,0,
            0,0,1,1,2,2,2,2,2,2,2,2,1,1,0,0,
            0,0,0,1,1,2,2,2,2,2,2,1,1,0,0,0,
            0,0,0,0,1,1,1,4,4,1,1,1,0,0,0,0,
            0,0,0,1,1,0,4,4,4,4,0,1,1,0,0,0,
            0,0,1,1,0,0,4,4,4,4,0,0,1,1,0,0,
        ];
        let palette = [
            RetroColors::TRANSPARENT,
            RetroColors::DARK_GRAY,
            RetroColors::BLUE,
            RetroColors::CYAN,
            RetroColors::ORANGE,
        ];
        Sprite::new(pattern.iter().map(|&i| palette[i as usize]).collect(), 16, 16)
    }

    // ── Enemies ──────────────────────────────────────────────────────────────

    pub fn enemy_grunt() -> Sprite {
        // 14x12 basic grunt
        #[rustfmt::skip]
        let pattern: [u8; 168] = [
            0,0,1,1,1,1,1,1,1,1,1,1,0,0,
            0,1,1,2,2,2,2,2,2,2,2,1,1,0,
            1,1,2,2,3,2,2,2,2,3,2,2,1,1,
            1,2,2,2,2,2,2,2,2,2,2,2,2,1,
            1,2,2,2,2,2,4,4,2,2,2,2,2,1,
            1,2,2,2,2,4,4,4,4,2,2,2,2,1,
            1,2,2,4,4,4,4,4,4,4,4,2,2,1,
            1,2,4,4,1,4,4,4,4,1,4,4,2,1,
            1,1,4,4,1,4,4,4,4,1,4,4,1,1,
            0,1,1,4,4,4,1,1,4,4,4,1,1,0,
            0,0,1,1,4,1,1,1,1,4,1,1,0,0,
            0,0,0,1,1,1,0,0,1,1,1,0,0,0,
        ];
        let palette = [
            RetroColors::TRANSPARENT,
            0xFF00CC00u32, // green outline
            0xFF009900u32, // dark green body
            RetroColors::RED, // red eyes
            RetroColors::BRIGHT_GREEN, // bright legs
        ];
        Sprite::new(pattern.iter().map(|&i| palette[i as usize]).collect(), 14, 12)
    }

    pub fn enemy_weaver() -> Sprite {
        // 14x12 zigzag weaver - purple/pink coloring
        #[rustfmt::skip]
        let pattern: [u8; 168] = [
            0,0,1,1,0,0,0,0,0,0,1,1,0,0,
            0,1,1,1,1,2,2,2,2,1,1,1,1,0,
            1,1,2,2,2,2,3,3,2,2,2,2,1,1,
            1,2,2,3,2,2,2,2,2,2,3,2,2,1,
            1,2,2,2,2,2,2,2,2,2,2,2,2,1,
            0,1,2,2,2,4,4,4,4,2,2,2,1,0,
            0,0,1,2,4,4,2,2,4,4,2,1,0,0,
            0,1,2,2,4,2,2,2,2,4,2,2,1,0,
            1,1,2,2,1,2,2,2,2,1,2,2,1,1,
            1,2,2,1,1,2,2,2,2,1,1,2,2,1,
            0,1,1,0,0,1,1,1,1,0,0,1,1,0,
            0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        ];
        let palette = [
            RetroColors::TRANSPARENT,
            0xFFCC00CCu32, // magenta outline
            0xFF880088u32, // dark magenta body
            RetroColors::YELLOW, // yellow eyes
            RetroColors::MAGENTA, // bright wings
        ];
        Sprite::new(pattern.iter().map(|&i| palette[i as usize]).collect(), 14, 12)
    }

    pub fn enemy_diver() -> Sprite {
        // 14x12 kamikaze diver - red/orange
        #[rustfmt::skip]
        let pattern: [u8; 168] = [
            0,0,0,0,0,0,1,1,0,0,0,0,0,0,
            0,0,0,0,0,1,2,2,1,0,0,0,0,0,
            0,0,0,0,1,2,2,2,2,1,0,0,0,0,
            0,0,1,1,2,2,3,3,2,2,1,1,0,0,
            0,1,1,2,2,2,2,2,2,2,2,1,1,0,
            1,1,2,2,2,2,4,4,2,2,2,2,1,1,
            1,2,2,2,4,4,4,4,4,4,2,2,2,1,
            1,2,4,4,4,2,4,4,2,4,4,4,2,1,
            1,1,4,4,2,2,4,4,2,2,4,4,1,1,
            0,1,1,4,2,2,2,2,2,2,4,1,1,0,
            0,0,1,1,2,2,2,2,2,2,1,1,0,0,
            0,0,0,1,1,1,1,1,1,1,1,0,0,0,
        ];
        let palette = [
            RetroColors::TRANSPARENT,
            RetroColors::DARK_RED,
            RetroColors::RED,
            RetroColors::YELLOW,
            RetroColors::ORANGE,
        ];
        Sprite::new(pattern.iter().map(|&i| palette[i as usize]).collect(), 14, 12)
    }

    pub fn enemy_boss() -> Sprite {
        // 48x24 boss enemy
        let mut pixels = vec![RetroColors::TRANSPARENT; 48 * 24];
        // Draw a large intimidating ship shape procedurally
        for py in 0..24u32 {
            for px in 0..48u32 {
                let cx = 24i32;
                let cy = 12i32;
                let dx = px as i32 - cx;
                let dy = py as i32 - cy;
                let dist = ((dx * dx + dy * dy) as f64).sqrt();

                let color = if dist < 8.0 {
                    RetroColors::CYAN // core
                } else if dist < 14.0 && (py < 20) {
                    if dx.abs() < 6 { RetroColors::BLUE } else { 0xFF0066FFu32 }
                } else if dist < 20.0 && py < 22 {
                    if (px + py) % 3 == 0 { RetroColors::DARK_BLUE } else { 0xFF004488u32 }
                } else if py < 10 && px > 4 && px < 44 && dist < 22.0 {
                    0xFF002255u32 // top hull
                } else {
                    RetroColors::TRANSPARENT
                };
                pixels[(py * 48 + px) as usize] = color;
            }
        }
        // Add red eyes
        for &(ex, ey) in &[(18u32, 8u32), (30u32, 8u32)] {
            for dy in 0..3u32 {
                for dx in 0..3u32 {
                    pixels[((ey + dy) * 48 + ex + dx) as usize] = RetroColors::RED;
                }
            }
        }
        Sprite::new(pixels, 48, 24)
    }

    // ── Projectiles ──────────────────────────────────────────────────────────

    pub fn player_bullet() -> Sprite {
        // 4x8 player bullet (yellow/white)
        #[rustfmt::skip]
        let pixels = vec![
            RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW,
            RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,
            RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,
            RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW,
            RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW,
            RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,
            RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,  RetroColors::WHITE,
            RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW, RetroColors::YELLOW,
        ];
        Sprite::new(pixels, 4, 8)
    }

    pub fn enemy_bullet() -> Sprite {
        // 4x4 enemy bullet (red/orange)
        let pixels = vec![
            RetroColors::ORANGE, RetroColors::RED, RetroColors::RED, RetroColors::ORANGE,
            RetroColors::RED,    RetroColors::ORANGE, RetroColors::ORANGE, RetroColors::RED,
            RetroColors::RED,    RetroColors::ORANGE, RetroColors::ORANGE, RetroColors::RED,
            RetroColors::ORANGE, RetroColors::RED, RetroColors::RED, RetroColors::ORANGE,
        ];
        Sprite::new(pixels, 4, 4)
    }

    // ── Explosion frames ─────────────────────────────────────────────────────

    pub fn explosion_frames() -> Vec<Sprite> {
        let sizes = [(8u32, RetroColors::YELLOW), (12, RetroColors::ORANGE),
                     (16, RetroColors::RED), (18, 0xFF882200u32),
                     (14, RetroColors::DARK_GRAY), (8, 0x44333333u32)];
        sizes.iter().map(|&(size, color)| {
            let mut pixels = vec![RetroColors::TRANSPARENT; (size * size) as usize];
            let cx = (size / 2) as i32;
            let cy = (size / 2) as i32;
            let r = (size as i32 / 2) - 1;
            for py in 0..size as i32 {
                for px in 0..size as i32 {
                    let dx = px - cx;
                    let dy = py - cy;
                    if dx * dx + dy * dy <= r * r {
                        // Irregular explosion pattern
                        let vary = ((px * 7 + py * 13) % 5) as u32;
                        let dist_sq = (dx * dx + dy * dy) as u32;
                        let inner = (r as u32 / 2).pow(2);
                        if dist_sq <= inner || vary < 3 {
                            pixels[(py as u32 * size + px as u32) as usize] = color;
                        }
                    }
                }
            }
            Sprite::new(pixels, size, size)
        }).collect()
    }

    // ── Power-up icons ───────────────────────────────────────────────────────

    /// Generate a simple 12x12 power-up icon with a letter indicator.
    pub fn powerup_sprite(color: u32) -> Sprite {
        let mut pixels = vec![RetroColors::TRANSPARENT; 12 * 12];
        // Draw border (top, bottom rows and left, right columns)
        for i in 0..12u32 {
            pixels[i as usize] = color;          // top row
            pixels[(11 * 12 + i) as usize] = color; // bottom row
            pixels[(i * 12) as usize] = color;      // left column
            pixels[(i * 12 + 11) as usize] = color; // right column
        }
        // Fill interior
        for py in 1..11u32 {
            for px in 1..11u32 {
                pixels[(py * 12 + px) as usize] = RetroColors::BLACK;
            }
        }
        Sprite::new(pixels, 12, 12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_sprite_dimensions() {
        let s = SpriteGenerator::player_ship();
        assert_eq!(s.width, 16);
        assert_eq!(s.height, 16);
        assert_eq!(s.pixels.len(), 256);
    }

    #[test]
    fn test_explosion_frames_count() {
        let frames = SpriteGenerator::explosion_frames();
        assert_eq!(frames.len(), 6);
    }

    #[test]
    fn test_animated_sprite_advances_frame() {
        let frames = vec![
            SpriteGenerator::player_ship(),
            SpriteGenerator::player_ship_thrust(),
        ];
        let mut anim = AnimatedSprite::new(frames, 0.1, true);
        assert_eq!(anim.current_frame, 0);
        anim.update(0.15);
        assert_eq!(anim.current_frame, 1);
        anim.update(0.15);
        assert_eq!(anim.current_frame, 0); // looped
    }

    #[test]
    fn test_animated_sprite_oneshot_finishes() {
        let frames = vec![
            SpriteGenerator::player_ship(),
            SpriteGenerator::player_ship_thrust(),
        ];
        let mut anim = AnimatedSprite::new(frames, 0.05, false);
        anim.update(0.2);
        assert!(anim.finished);
        assert_eq!(anim.current_frame, 1);
    }
}
