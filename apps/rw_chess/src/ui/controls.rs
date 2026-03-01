use leptos::prelude::*;
use crate::state::{game::GameState};

/// Thin controls bar below the board.
#[component]
pub fn Controls(on_new_game: Callback<()>) -> impl IntoView {
    let game = expect_context::<GameState>();

    let difficulty_label = move || game.difficulty.get().label();
    let _is_thinking = move || game.engine_highlight.get().is_none()
        && !game.active_color.get_untracked().eq(&game.player_color.get_untracked());

    view! {
        <div class="controls">
            <div class="info-section">
                <h3>"Info"</h3>
                <div style="font-size:0.85rem;">
                    <span style="color:var(--text-dim)">"Difficulty: "</span>
                    <strong>{difficulty_label}</strong>
                </div>
                <div style="font-size:0.85rem; margin-top:0.3rem;">
                    <span style="color:var(--text-dim)">"Player: "</span>
                    <strong>{move || game.player_name.get()}</strong>
                </div>
            </div>
            <button class="btn" on:click=move |_| on_new_game.run(())>
                "⚙ New Game"
            </button>
        </div>
    }
}
