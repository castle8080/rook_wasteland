use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_timers::future::TimeoutFuture;

use crate::{
    engine::{
        persona::{
            engine_capture_event, get_commentary, persona_for_difficulty,
            player_capture_event, CommentaryEvent,
        },
        search::best_move,
    },
    rules::validation::is_in_check,
    state::{
        game::GameState,
        piece::GamePhase,
    },
    ui::{
        board::BoardView,
        commentary_box::CommentaryBox,
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

    let in_setup = RwSignal::new(true);

    let on_start = {
        let game = game.clone();
        Callback::new(move |config: SetupConfig| {
            game.reset();
            game.player_name.set(config.player_name);
            game.player_color.set(config.player_color);
            game.difficulty.set(config.difficulty);
            // Set persona from chosen difficulty
            game.persona.set(persona_for_difficulty(config.difficulty));
            in_setup.set(false);

            // Greeting commentary
            let persona = game.persona.get_untracked();
            if let Some(line) = get_commentary(persona.id, CommentaryEvent::GameStart) {
                game.set_commentary(line);
            }

        })
    };

    let on_new_game = Callback::new(move |()| {
        in_setup.set(true);
    });

    let on_rematch = {
        let game = game.clone();
        Callback::new(move |()| {
            let player_color = game.player_color.get();
            let player_name = game.player_name.get();
            let difficulty = game.difficulty.get();
            game.reset();
            game.player_name.set(player_name);
            game.player_color.set(player_color);
            game.difficulty.set(difficulty);
            game.persona.set(persona_for_difficulty(difficulty));

            // Rematch greeting
            let persona = game.persona.get_untracked();
            if let Some(line) = get_commentary(persona.id, CommentaryEvent::GameStart) {
                game.set_commentary(line);
            }
            // The reactive Effect handles triggering the engine move when it's the engine's turn.
        })
    };

    // Watch for engine's turn
    let game_for_effect = game.clone();
    Effect::new(move |_| {
        let active = game_for_effect.active_color.get();
        let player = game_for_effect.player_color.get();
        let phase = game_for_effect.phase.get();
        let is_setup = in_setup.get();

        if is_setup || active == player {
            return;
        }

        if matches!(phase, GamePhase::Playing | GamePhase::Check) {
            trigger_engine_move(game_for_effect.clone());
        } else if matches!(phase, GamePhase::Checkmate | GamePhase::Stalemate | GamePhase::DrawFiftyMove) {
            // It's the engine's turn but the game is already over — the player just won or drew.
            let persona = game_for_effect.persona.get_untracked();
            let event = if matches!(phase, GamePhase::Checkmate) {
                CommentaryEvent::EngineLoses
            } else {
                CommentaryEvent::Stalemate
            };
            if let Some(line) = get_commentary(persona.id, event) {
                game_for_effect.set_commentary(line);
            }
        }
    });

    view! {
        <div class="app">
            <header class="app-header">
                <h1>"♜ Rook Wasteland"</h1>
            </header>

            <Show
                when=move || !in_setup.get()
                fallback=move || view! { <SetupScreen on_start=on_start /> }
            >
                <div class="game-layout">
                    <div style="display:flex; flex-direction:column; gap:1rem;">
                        <BoardView />
                        <CommentaryBox />
                    </div>
                    <div style="display:flex; flex-direction:column; gap:1rem;">
                        <InfoPanel />
                        <Controls on_new_game=on_new_game />
                    </div>
                </div>
                <GameOverOverlay on_rematch=on_rematch on_new_game=on_new_game />
            </Show>
        </div>
    }
}

/// Spawn an async task to compute and apply the engine's move, with commentary.
pub fn trigger_engine_move(game: GameState) {
    spawn_local(async move {
        // Brief delay so UI updates first
        TimeoutFuture::new(150).await;

        // Guard: verify it is still the engine's turn after the async delay.
        // State can change between the call site and here (e.g. a duplicate
        // trigger_engine_move call resolves first and advances active_color).
        {
            use crate::state::game::engine_should_move;
            let active = game.active_color.get_untracked();
            let player = game.player_color.get_untracked();
            let phase  = game.phase.get_untracked();
            if !engine_should_move(active, player, phase) {
                leptos::logging::error!(
                    "[trigger_engine_move] Unexpected: called when it is not the engine's turn \
                     (active={active:?}, player={player:?}, phase={phase:?}). Bailing."
                );
                return;
            }
        }

        let board = game.board.get_untracked();
        let color = game.active_color.get_untracked();
        let ep = game.en_passant.get_untracked();
        let castling = game.castling.get_untracked();
        let depth = game.difficulty.get_untracked().search_depth();
        let persona = game.persona.get_untracked();
        let last_move = game.last_move.get_untracked();

        // ── React to what the player just did (before engine moves) ──────────
        if let Some(_lm) = last_move {
            // Check if the player's last move gave check to the engine
            if is_in_check(&board, color) {
                if let Some(line) = get_commentary(persona.id, CommentaryEvent::PlayerGivesCheck) {
                    game.set_commentary(line);
                    TimeoutFuture::new(1200).await;
                }
            }
            // Check if the player captured an engine piece
            else if let Some(captured) = game.move_history.get_untracked().last()
                .and_then(|r| r.captured)
                .filter(|p| p.color == color) // engine's piece was captured
            {
                let event = player_capture_event(captured.kind);
                if let Some(line) = get_commentary(persona.id, event) {
                    game.set_commentary(line);
                    TimeoutFuture::new(1000).await;
                }
            }
        }

        // ── Compute engine move ───────────────────────────────────────────────
        let mv = best_move(&board, color, depth, ep, castling);

        if let Some(mv) = mv {
            // Determine commentary event based on what's about to happen
            let captured_piece = board.get(mv.to)
                .or_else(|| if mv.is_en_passant {
                    let cap_pos = crate::state::piece::Pos::new(mv.to.file, mv.from.rank);
                    board.get(cap_pos)
                } else { None });

            game.apply_move(mv);

            let new_board = game.board.get_untracked();
            let new_phase = game.phase.get_untracked();
            let player_color = game.player_color.get_untracked();

            let commentary_event = if matches!(new_phase, GamePhase::Checkmate) {
                Some(CommentaryEvent::EngineWins)
            } else if matches!(new_phase, GamePhase::Stalemate | GamePhase::DrawFiftyMove) {
                Some(CommentaryEvent::Stalemate)
            } else if is_in_check(&new_board, player_color) {
                Some(CommentaryEvent::EngineGivesCheck)
            } else if let Some(cap) = captured_piece {
                Some(engine_capture_event(cap.kind))
            } else {
                Some(CommentaryEvent::EngineMoveGeneral)
            };

            if let Some(event) = commentary_event
                && let Some(line) = get_commentary(persona.id, event) {
                    game.set_commentary(line);
                }

            // Set engine highlight and clear after 1.5s
            game.engine_highlight.set(Some(mv));
            TimeoutFuture::new(1500).await;
            game.engine_highlight.set(None);
        } else {
            // No legal moves for engine — game over (should have been caught in apply_move)
            let phase = game.phase.get_untracked();
            let event = if matches!(phase, GamePhase::Checkmate) {
                CommentaryEvent::EngineLoses
            } else {
                CommentaryEvent::Stalemate
            };
            if let Some(line) = get_commentary(persona.id, event) {
                game.set_commentary(line);
            }
        }
    });
}
