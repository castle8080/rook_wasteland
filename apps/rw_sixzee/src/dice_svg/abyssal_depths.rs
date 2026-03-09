//! Abyssal Depths die faces — bioluminescent jellyfish.
//!
//! Each pip is a jellyfish silhouette: a domed ellipse bell with three
//! sinuous tentacles rendered as quadratic-bezier `<path>` elements.  All
//! shapes use `var(--color-accent)` so they pick up the teal/aqua palette
//! defined by the `[data-theme="abyssal_depths"]` CSS block.

use leptos::prelude::*;

use crate::dice_svg::pip_positions;

/// Render a single pip as a bioluminescent jellyfish.
///
/// The jellyfish consists of a domed ellipse bell positioned slightly above
/// the pip centre, and three quadratic-bezier tentacles drooping from its
/// base.  All geometry is sized to be legible at the 56×56 px die-button
/// scale while remaining recognisable in the 20×20 px Settings preview.
fn pip(cx: f32, cy: f32) -> impl IntoView {
    // Bell — centred 2 units above pip centre so tentacles balance it down.
    let bell_cx = format!("{cx:.1}");
    let bell_cy = format!("{:.1}", cy - 2.0);

    // Tentacle anchor row: base of the bell.
    let ty0 = cy + 2.0; // start y (bell base)
    let ty1 = cy + 5.5; // control y (mid-curve)
    let ty2 = cy + 9.0; // end y (tentacle tip)

    // Left tentacle: curves left then back right.
    let tl = format!(
        "M {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1}",
        cx - 3.0,
        ty0,
        cx - 5.5,
        ty1,
        cx - 3.0,
        ty2,
    );
    // Centre tentacle: curves slightly right for visual asymmetry.
    let tc = format!(
        "M {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1}",
        cx,
        ty0,
        cx + 2.5,
        ty1,
        cx,
        ty2,
    );
    // Right tentacle: curves right then back left.
    let tr = format!(
        "M {:.1},{:.1} Q {:.1},{:.1} {:.1},{:.1}",
        cx + 3.0,
        ty0,
        cx + 5.5,
        ty1,
        cx + 3.0,
        ty2,
    );

    view! {
        <g>
            <ellipse cx={bell_cx} cy={bell_cy} rx="5" ry="4"
                     fill="var(--color-accent)" />
            <path d={tl} stroke="var(--color-accent)"
                  style="stroke-width:1.2;fill:none;" />
            <path d={tc} stroke="var(--color-accent)"
                  style="stroke-width:1.2;fill:none;" />
            <path d={tr} stroke="var(--color-accent)"
                  style="stroke-width:1.2;fill:none;" />
        </g>
    }
}

/// Render an SVG die face for Abyssal Depths (values 1–6).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dice_svg::pip_positions;

    /// Each call to `face(v)` renders exactly `v` jellyfish pips — verified
    /// indirectly by checking that `pip_positions` returns the right count,
    /// which is the sole input driving pip rendering.
    #[test]
    fn pip_count_matches_value() {
        for v in 1u8..=6 {
            assert_eq!(
                pip_positions(v).len(),
                v as usize,
                "pip_positions count for value {v}"
            );
        }
    }

    /// Values outside 1–6 produce an empty pip list → blank SVG face.
    #[test]
    fn out_of_range_values_produce_no_pips() {
        assert!(pip_positions(0).is_empty());
        assert!(pip_positions(7).is_empty());
    }

    /// Tentacle tip y-coordinate must stay within the 100-unit viewBox for
    /// all canonical pip positions (tallest pip is at cy = 75).
    #[test]
    fn tentacle_tips_stay_within_viewbox() {
        // ty2 = cy + 9.0; worst case cy = 75 → tip = 84 < 100
        for &(_cx, cy) in pip_positions(6) {
            let tip_y = cy + 9.0;
            assert!(
                tip_y < 100.0,
                "tentacle tip y={tip_y:.1} exceeds 100-unit viewBox for pip cy={cy}"
            );
        }
    }
}
