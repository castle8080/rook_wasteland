use leptos::prelude::*;
use crate::{
    state::{game::GameState, piece::{Pos}},
    ui::piece::PieceView,
};

/// Renders a single board square.
#[component]
pub fn SquareView(
    pos: Pos,
    #[prop(into)] flipped: bool,
) -> impl IntoView {
    let game = expect_context::<GameState>();

    let is_light = (pos.file + pos.rank) % 2 == 1;
    let base_class = if is_light { "square light" } else { "square dark" };

    let g1 = game.clone();
    let square_class = move || {
        let mut cls = base_class.to_string();

        let selected = g1.selected_square.get();
        let last = g1.last_move.get();
        let engine_hl = g1.engine_highlight.get();
        let board = g1.board.get();
        let phase = g1.phase.get();
        let active = g1.active_color.get();

        if selected == Some(pos) {
            cls.push_str(" selected");
        }
        if let Some(m) = last {
            if m.from == pos { cls.push_str(" last-move-from"); }
            else if m.to == pos { cls.push_str(" last-move-to"); }
        }
        if let Some(m) = engine_hl {
            if m.from == pos || m.to == pos {
                cls.push_str(" engine-highlight");
            }
        }
        if matches!(phase, crate::state::piece::GamePhase::Check) {
            if let Some(king_pos) = board.find_king(active) {
                if king_pos == pos { cls.push_str(" in-check"); }
            }
        }
        cls
    };

    let g2 = game.clone();
    let g3 = game.clone();
    let g4 = game.clone();
    let g5 = game.clone();

    let is_valid_target = move || g2.is_valid_target(pos);
    let has_piece_fn = move || g3.board.get().get(pos).is_some();
    let on_click = move |_| handle_click(&g4, pos);

    view! {
        <div class=square_class on:click=on_click>
            <Show when=is_valid_target>
                <div class=move || {
                    if has_piece_fn() { "valid-capture-ring".to_string() }
                    else { "valid-move-dot".to_string() }
                }>
                    <div class="valid-move-dot" />
                </div>
            </Show>
            {move || {
                g5.board.get().get(pos).map(|piece| view! { <PieceView piece=piece /> })
            }}
        </div>
    }
}

fn handle_click(game: &GameState, pos: Pos) {
    if game.is_game_over() {
        return;
    }
    let selected = game.selected_square.get();
    if selected.is_some() {
        if game.try_move_to(pos) {
            return;
        }
    }
    game.select_square(pos);
}

