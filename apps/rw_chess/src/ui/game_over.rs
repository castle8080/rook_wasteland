use leptos::prelude::*;
use crate::state::{game::GameState, piece::GamePhase};

/// Game over overlay shown when the game ends.
#[component]
pub fn GameOverOverlay(on_rematch: Callback<()>, on_new_game: Callback<()>) -> impl IntoView {
    let game = expect_context::<GameState>();

    let result_text = move || {
        let phase = game.phase.get();
        let active = game.active_color.get(); // the side that couldn't move
        let player_color = game.player_color.get();
        let player_name = game.player_name.get();

        match phase {
            GamePhase::Checkmate => {
                let winner = active.opposite();
                if winner == player_color {
                    format!("🏆 {player_name} wins by Checkmate!")
                } else {
                    "🤖 Engine wins by Checkmate!".to_string()
                }
            }
            GamePhase::Stalemate => "½ Draw — Stalemate!".to_string(),
            GamePhase::DrawFiftyMove => "½ Draw — 50-move rule!".to_string(),
            _ => String::new(),
        }
    };

    let phase_is_over = move || {
        matches!(
            game.phase.get(),
            GamePhase::Checkmate | GamePhase::Stalemate | GamePhase::DrawFiftyMove
        )
    };

    view! {
        <Show when=phase_is_over>
            <div class="game-over-overlay">
                <div class="game-over-card">
                    <h2>"Game Over"</h2>
                    <p>{result_text}</p>
                    <div class="button-group">
                        <button
                            class="btn btn-primary"
                            on:click=move |_| on_rematch.run(())
                        >
                            "♻ Rematch"
                        </button>
                        <button
                            class="btn"
                            on:click=move |_| on_new_game.run(())
                        >
                            "⚙ New Game"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
