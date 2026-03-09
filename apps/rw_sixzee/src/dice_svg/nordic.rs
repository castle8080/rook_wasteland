//! Nordic Minimal die faces — clean geometric dots (filled circles).
//!
//! Stark, precise; pip positions match the canonical layout exactly.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Render a single pip as a filled circle.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    let cxs = format!("{cx}");
    let cys = format!("{cy}");
    view! {
        <circle cx={cxs} cy={cys} r="7" fill="var(--color-accent)" />
    }
}

/// Render an SVG die face for Nordic Minimal (values 1–6).
pub fn face(value: u8) -> impl IntoView {
    let pips: Vec<_> = pip_positions(value)
        .iter()
        .map(|&(cx, cy)| pip(cx, cy).into_any())
        .collect();
    view! {
        <svg viewBox="0 0 100 100" width="100%" height="100%"
             style="display: block;">
            {pips}
        </svg>
    }
}
