//! Dice row component — renders 5 dice with hold/unhold toggle.
//!
//! Reads `RwSignal<GameState>` and `ActiveTheme` from context. Each die shows
//! its themed SVG face when rolled or `?` when unrolled. Clicking a die toggles
//! its held state. Held dice display with a double border
//! (`.dice-row__die--held`).

use leptos::prelude::*;

use crate::dice_svg::DiceFace;
use crate::state::game::GameState;
use crate::state::ActiveTheme;

/// Renders the 5-die row. Reads `game_signal` and `ActiveTheme` from context.
///
/// Die clicks are no-ops when the die is unrolled (`None`).
#[component]
pub fn DiceRow() -> impl IntoView {
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");
    let active_theme =
        use_context::<ActiveTheme>().expect("ActiveTheme context must be provided");

    let dice_views = move || {
        let state = game_signal.get();
        let theme = active_theme.0.get();
        (0..5_usize)
            .map(|i| {
                let value = state.dice[i];
                let held = state.held[i];
                let rolled = value.is_some();

                let aria_label = format!(
                    "Die {} value {} {}",
                    i + 1,
                    value.map_or_else(|| "?".to_string(), |v| v.to_string()),
                    if held { "held" } else { "" }
                );

                let class = if !rolled {
                    "dice-row__die dice-row__die--unrolled"
                } else if held {
                    "dice-row__die dice-row__die--held"
                } else {
                    "dice-row__die"
                };

                let on_click = move |_| {
                    if rolled {
                        game_signal.update(|s| {
                            s.held[i] = !s.held[i];
                        });
                    }
                };

                // Show SVG face for rolled dice; plain "?" for unrolled.
                let face_content = match value {
                    Some(v) => view! { <DiceFace theme=theme value=v /> }.into_any(),
                    None => view! { "?" }.into_any(),
                };

                view! {
                    <button
                        class=class
                        on:click=on_click
                        aria-label=aria_label
                    >
                        {face_content}
                    </button>
                }
            })
            .collect_view()
    };

    view! {
        <div class="dice-row">
            {dice_views}
        </div>
    }
}
