//! Devil Rock die faces — 5-pointed star (pentagram) symbols.
//!
//! Each pip is a filled 10-point star polygon.  Using alternating outer/inner
//! radius points produces a correct star shape without needing `fill-rule`.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Compute the SVG `points` string for a 10-point star (pentagram) centred at
/// `(cx, cy)` with outer radius `r_outer` and inner radius `r_inner`.
fn star_points(cx: f32, cy: f32, r_outer: f32, r_inner: f32) -> String {
    use std::f32::consts::PI;
    let mut parts = Vec::with_capacity(10);
    for i in 0..5 {
        let outer_angle = -PI / 2.0 + i as f32 * 2.0 * PI / 5.0;
        let inner_angle = outer_angle + PI / 5.0;
        parts.push(format!(
            "{:.2},{:.2}",
            cx + r_outer * outer_angle.cos(),
            cy + r_outer * outer_angle.sin()
        ));
        parts.push(format!(
            "{:.2},{:.2}",
            cx + r_inner * inner_angle.cos(),
            cy + r_inner * inner_angle.sin()
        ));
    }
    parts.join(" ")
}

/// Render a single pip as a 5-pointed star.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    let pts = star_points(cx, cy, 8.0, 3.5);
    view! {
        <polygon points={pts} fill="var(--color-accent)" />
    }
}

/// Render an SVG die face for Devil Rock (values 1–6).
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
