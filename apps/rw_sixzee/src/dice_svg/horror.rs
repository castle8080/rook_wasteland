//! Horror die faces — stylised skulls (circle with hollow eye sockets).
//!
//! Each pip is a filled circle with two smaller hollow circles cut out to
//! suggest eye sockets — a minimal but instantly recognisable skull motif.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Render a single pip as a simplified skull.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    // Head — slightly offset upward from centre so the skull looks balanced.
    let cxs = format!("{cx}");
    let cys = format!("{:.1}", cy - 0.5);
    let le_x = format!("{:.1}", cx - 2.6);
    let re_x = format!("{:.1}", cx + 2.6);
    let eye_y = format!("{:.1}", cy - 1.5);
    view! {
        <g>
            <circle cx={cxs} cy={cys} r="7" fill="var(--color-accent)" />
            <circle cx={le_x} cy={eye_y.clone()} r="1.8"
                    fill="var(--color-surface)" />
            <circle cx={re_x} cy={eye_y} r="1.8"
                    fill="var(--color-surface)" />
        </g>
    }
}

/// Render an SVG die face for Horror (values 1–6).
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
