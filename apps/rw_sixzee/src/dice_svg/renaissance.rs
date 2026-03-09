//! Renaissance die faces — illuminated rosettes.
//!
//! Each pip is a gilded rosette: a filled centre circle surrounded by a
//! stroked outer ring, evoking illuminated manuscript flourishes.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Render a single pip as an illuminated rosette.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    let cxs = format!("{cx}");
    let cys = format!("{cy}");
    view! {
        <g>
            <circle cx={cxs.clone()} cy={cys.clone()} r="7"
                    fill="none" stroke="var(--color-accent)"
                    style="stroke-width: 2;" />
            <circle cx={cxs} cy={cys} r="3" fill="var(--color-accent)" />
        </g>
    }
}

/// Render an SVG die face for Renaissance (values 1–6).
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
