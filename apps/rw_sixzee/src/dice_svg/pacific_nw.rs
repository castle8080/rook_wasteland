//! Pacific Northwest die faces — cedar tree-ring symbols.
//!
//! Each pip is a set of three concentric circles evoking the annual growth
//! rings of a Pacific Northwest cedar tree, with a solid centre.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Render a single pip as a cedar ring symbol.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    let cxs = format!("{cx}");
    let cys = format!("{cy}");
    view! {
        <g>
            <circle cx={cxs.clone()} cy={cys.clone()} r="7.5"
                    fill="none" stroke="var(--color-accent)"
                    style="stroke-width: 1.5;" />
            <circle cx={cxs.clone()} cy={cys.clone()} r="4.5"
                    fill="none" stroke="var(--color-accent)"
                    style="stroke-width: 1.5;" />
            <circle cx={cxs} cy={cys} r="1.8" fill="var(--color-accent)" />
        </g>
    }
}

/// Render an SVG die face for Pacific Northwest (values 1–6).
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
