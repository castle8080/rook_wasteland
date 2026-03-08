//! Dice row component — renders 5 dice with hold/unhold toggle.
//!
//! Reads `RwSignal<GameState>` from context. Each die shows its current value
//! or `?` when unrolled. Clicking a die toggles its held state. Held dice
//! display with a double border (`.dice-row__die--held`).

use leptos::prelude::*;

use crate::state::game::GameState;

/// Renders the 5-die row. Reads `game_signal` from context.
///
/// Die clicks are no-ops when the die is unrolled (`None`).
#[component]
pub fn DiceRow() -> impl IntoView {
    let game_signal =
        use_context::<RwSignal<GameState>>().expect("game_signal context must be provided");

    let dice_views = move || {
        let state = game_signal.get();
        (0..5_usize)
            .map(|i| {
                let value = state.dice[i];
                let held = state.held[i];
                let rolled = value.is_some();

                let label = match value {
                    Some(v) => v.to_string(),
                    None => "?".to_string(),
                };
                let aria_label = format!(
                    "Die {} value {} {}",
                    i + 1,
                    label,
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

                view! {
                    <button
                        class=class
                        on:click=on_click
                        aria-label=aria_label
                    >
                        {label}
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
