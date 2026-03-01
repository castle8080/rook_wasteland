/// Classic 16-color CGA/arcade palette in RGBA8888 (0xAABBGGRR format
/// where AA=alpha, BB=blue, GG=green, RR=red).
///
/// NOTE: The pixel format used in SpriteGenerator stores colors as 0xAARRGGBB
/// (high byte = alpha, then R, G, B) to match the `rgba(r,g,b,a)` CSS extraction
/// used in Sprite::draw_optimized.
pub struct RetroColors;

#[allow(dead_code)]
impl RetroColors {
    // Format: 0xAARRGGBB
    pub const TRANSPARENT: u32 = 0x00000000;
    pub const BLACK: u32       = 0xFF000000;
    pub const DARK_BLUE: u32   = 0xFF0000AA;
    pub const DARK_GREEN: u32  = 0xFF00AA00;
    pub const DARK_CYAN: u32   = 0xFF00AAAA;
    pub const DARK_RED: u32    = 0xFFAA0000;
    pub const DARK_MAGENTA: u32= 0xFFAA00AA;
    pub const BROWN: u32       = 0xFFAA5500;
    pub const GRAY: u32        = 0xFFAAAAAA;
    pub const DARK_GRAY: u32   = 0xFF555555;
    pub const BLUE: u32        = 0xFF5555FF;
    pub const GREEN: u32       = 0xFF55FF55;
    pub const CYAN: u32        = 0xFF55FFFF;
    pub const RED: u32         = 0xFFFF5555;
    pub const MAGENTA: u32     = 0xFFFF55FF;
    pub const YELLOW: u32      = 0xFFFFFF55;
    pub const WHITE: u32       = 0xFFFFFFFF;

    // Extra game colors
    pub const ORANGE: u32      = 0xFFFF8800;
    pub const LIGHT_BLUE: u32  = 0xFF88CCFF;
    pub const BRIGHT_GREEN: u32= 0xFF00FF00;
}

/// Convert a packed 0xAARRGGBB color to a CSS `rgba(r,g,b,a)` string.
pub fn color_to_css(color: u32) -> String {
    let a = ((color >> 24) & 0xFF) as f64 / 255.0;
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;
    format!("rgba({},{},{},{:.3})", r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_to_css_white() {
        let s = color_to_css(RetroColors::WHITE);
        assert!(s.starts_with("rgba(255,255,255,"));
    }

    #[test]
    fn test_color_to_css_transparent() {
        let s = color_to_css(RetroColors::TRANSPARENT);
        assert!(s.contains(",0.000)"));
    }
}
