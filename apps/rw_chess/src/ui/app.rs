use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_timers::future::TimeoutFuture;

use crate::{
    engine::search::best_move,
    state::{
        game::GameState,
        piece::{Color, GamePhase},
    },
    ui::{
        board::BoardView,
        controls::Controls,
        game_over::GameOverOverlay,
        info_panel::InfoPanel,
        setup::{SetupConfig, SetupScreen},
    },
};

#[component]
pub fn App() -> impl IntoView {
    let game = GameState::new();
    provide_context(game.clone());

    // Whether we're on the setup screen
    let in_setup = RwSignal::new(true);

    let on_start = {
        let game = game.clone();
        Callback::new(move |config: SetupConfig| {
            game.reset();
            game.player_name.set(config.player_name);
            game.player_color.set(config.player_color);
            game.difficulty.set(config.difficulty);
            in_setup.set(false);

            // If player chose Black, engine (White) moves first
            if config.player_color == Color::Black {
                trigger_engine_move(game.clone());
            }
        })
    };

    let on_new_game = Callback::new(move |()| {
        in_setup.set(true);
    });

    let on_rematch = {
        let game = game.clone();
        let on_new_game = on_new_game.clone();
        Callback::new(move |()| {
            let player_color = game.player_color.get();
            let player_name = game.player_name.get();
            let difficulty = game.difficulty.get();
            game.reset();
            game.player_name.set(player_name);
            game.player_color.set(player_color);
            game.difficulty.set(difficulty);
            // If player is Black, engine goes first
            if player_color == Color::Black {
                trigger_engine_move(game.clone());
            }
        })
    };

    // Watch for engine's turn and trigger engine move
    let game_for_effect = game.clone();
    Effect::new(move |_| {
        let active = game_for_effect.active_color.get();
        let player = game_for_effect.player_color.get();
        let phase = game_for_effect.phase.get();
        let is_setup = in_setup.get();

        if !is_setup
            && active != player
            && matches!(phase, GamePhase::Playing | GamePhase::Check)
        {
            trigger_engine_move(game_for_effect.clone());
        }
    });

    view! {
        <div class="app">
            <header class="app-header">
                <h1>"♜ Rook Wasteland"</h1>
            </header>

            <Show
                when=move || !in_setup.get()
                fallback=move || view! { <SetupScreen on_start=on_start.clone() /> }
            >
                <div class="game-layout">
                    <BoardView />
                    <div style="display:flex; flex-direction:column; gap:1rem;">
                        <InfoPanel />
                        <Controls on_new_game=on_new_game.clone() />
                        <EngineThinkingIndicator />
                    </div>
                </div>
                <GameOverOverlay on_rematch=on_rematch.clone() on_new_game=on_new_game.clone() />
            </Show>
        </div>
    }
}

/// Renders "Engine is thinking..." while the engine is computing.
#[component]
fn EngineThinkingIndicator() -> impl IntoView {
    let game = expect_context::<GameState>();
    let is_thinking = RwSignal::new(false);

    // We expose this signal on the game context indirectly — just track via signals
    view! {
        <Show when=move || {
            let active = game.active_color.get();
            let player = game.player_color.get();
            let phase = game.phase.get();
            active != player && matches!(phase, GamePhase::Playing | GamePhase::Check)
        }>
            <div class="thinking-indicator">
                "🤖 Engine is thinking"
                <span class="thinking-dots">
                    <span>"."</span><span>"."</span><span>"."</span>
                </span>
            </div>
        </Show>
    }
}

/// Spawn an async task to compute and apply the engine's move.
pub fn trigger_engine_move(game: GameState) {
    spawn_local(async move {
        // Brief artificial delay so UI updates first (shows "thinking")
        TimeoutFuture::new(150).await;

        let board = game.board.get_untracked();
        let color = game.active_color.get_untracked();
        let ep = game.en_passant.get_untracked();
        let castling = game.castling.get_untracked();
        let depth = game.difficulty.get_untracked().search_depth();

        let mv = best_move(&board, color, depth, ep, castling);

        if let Some(mv) = mv {
            // Apply the move
            game.apply_move(mv);

            // Set engine highlight and clear after 1.5s
            game.engine_highlight.set(Some(mv));
            TimeoutFuture::new(1500).await;
            game.engine_highlight.set(None);
        }
    });
}
