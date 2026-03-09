//! Borg die faces — hexagonal circuit-node squares with a hollow centre.
//!
//! Each pip is a small filled square (representing a data block / circuit pad)
//! with a circular void at the centre to evoke a circuit node.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Render a single pip as a circuit-node square.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    let x = format!("{:.1}", cx - 5.5);
    let y = format!("{:.1}", cy - 5.5);
    let cxs = format!("{cx}");
    let cys = format!("{cy}");
    view! {
        <g>
            <rect x={x} y={y} width="11" height="11"
                  fill="var(--color-accent)" rx="1" />
            <circle cx={cxs} cy={cys} r="2.5" fill="var(--color-surface)" />
        </g>
    }
}

/// Render an SVG die face for Borg (values 1–6).
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
