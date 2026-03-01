use leptos::prelude::*;
use crate::state::piece::{Color, Piece, PieceKind, Promotion};

/// Renders a chess piece glyph.
#[component]
pub fn PieceView(piece: Piece) -> impl IntoView {
    let color_class = match piece.color {
        Color::White => "white",
        Color::Black => "black",
    };
    view! {
        <span class=format!("piece {}", color_class)>
            {piece.glyph()}
        </span>
    }
}

/// Promotion dialog shown when a pawn reaches the back rank.
#[component]
pub fn PromotionDialog(
    color: Color,
    on_choose: Callback<Promotion>,
) -> impl IntoView {
    let choices = [
        (Promotion::Queen,  Piece::new(PieceKind::Queen,  color)),
        (Promotion::Rook,   Piece::new(PieceKind::Rook,   color)),
        (Promotion::Bishop, Piece::new(PieceKind::Bishop, color)),
        (Promotion::Knight, Piece::new(PieceKind::Knight, color)),
    ];

    view! {
        <div class="promotion-dialog">
            <div class="promotion-choices">
                {choices.into_iter().map(|(promo, piece)| {
                    view! {
                        <div
                            class="promotion-choice"
                            on:click=move |_| on_choose.run(promo)
                        >
                            <PieceView piece=piece />
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
