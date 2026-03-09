//! SVG dice face components, one module per theme (M8).
//!
//! ## Structure
//!
//! - `DiceFace(theme, value)` — top-level component; dispatches to the correct
//!   per-theme `face(value)` function.
//! - Six sub-modules (`nordic`, `abyssal_depths`, `borg`, `horror`, `renaissance`,
//!   `pacific_nw`) each export `pub fn face(value: u8) -> impl IntoView`.
//! - `pip_positions(value)` — shared helper returning the canonical (x, y)
//!   centres for pips 1–6 on a 100×100 viewBox.
//!
//! ## SVG coordinate system
//!
//! All faces use `viewBox="0 0 100 100"`.  Canonical pip positions:
//!
//! ```text
//! TL(28,25)  TR(72,25)
//! ML(28,50)  MR(72,50)
//! BL(28,75)  BR(72,75)
//!        C(50,50)
//! ```

use leptos::prelude::*;

use crate::state::Theme;

pub mod abyssal_depths;
pub mod borg;
pub mod horror;
pub mod nordic;
pub mod pacific_nw;
pub mod renaissance;

// ── Shared pip-position table ──────────────────────────────────────────────

const PIP_1: [(f32, f32); 1] = [(50.0, 50.0)];
const PIP_2: [(f32, f32); 2] = [(72.0, 25.0), (28.0, 75.0)];
const PIP_3: [(f32, f32); 3] = [(72.0, 25.0), (50.0, 50.0), (28.0, 75.0)];
const PIP_4: [(f32, f32); 4] = [(28.0, 25.0), (72.0, 25.0), (28.0, 75.0), (72.0, 75.0)];
const PIP_5: [(f32, f32); 5] = [
    (28.0, 25.0),
    (72.0, 25.0),
    (50.0, 50.0),
    (28.0, 75.0),
    (72.0, 75.0),
];
const PIP_6: [(f32, f32); 6] = [
    (28.0, 25.0),
    (72.0, 25.0),
    (28.0, 50.0),
    (72.0, 50.0),
    (28.0, 75.0),
    (72.0, 75.0),
];

/// Returns the canonical (x, y) pip positions for a die face value 1–6.
///
/// Uses a 100×100 coordinate space.  Returns an empty slice for values
/// outside 1–6.
pub fn pip_positions(value: u8) -> &'static [(f32, f32)] {
    match value {
        1 => &PIP_1,
        2 => &PIP_2,
        3 => &PIP_3,
        4 => &PIP_4,
        5 => &PIP_5,
        6 => &PIP_6,
        _ => &[],
    }
}

// ── DiceFace component ─────────────────────────────────────────────────────

/// Renders an SVG die face for the given theme and value (1–6).
///
/// The SVG uses `width="100%" height="100%"` so it fills whatever container
/// it is placed in (a `56×56px` button in the dice row, or a larger preview
/// card in Settings).  All colours are CSS custom properties so the face
/// inherits colours from the nearest `data-theme` ancestor.
#[component]
pub fn DiceFace(
    /// The active visual theme — determines which symbol set to draw.
    theme: Theme,
    /// Die value to display (1–6).  Values outside this range render empty.
    value: u8,
) -> impl IntoView {
    match theme {
        Theme::NordicMinimal => nordic::face(value).into_any(),
        Theme::AbyssalDepths => abyssal_depths::face(value).into_any(),
        Theme::Borg => borg::face(value).into_any(),
        Theme::Horror => horror::face(value).into_any(),
        Theme::Renaissance => renaissance::face(value).into_any(),
        Theme::PacificNorthwest => pacific_nw::face(value).into_any(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pip_positions_count_matches_value() {
        for v in 1u8..=6 {
            assert_eq!(pip_positions(v).len(), v as usize, "pip count for value {v}");
        }
    }

    #[test]
    fn pip_positions_out_of_range_is_empty() {
        assert!(pip_positions(0).is_empty());
        assert!(pip_positions(7).is_empty());
    }
}
