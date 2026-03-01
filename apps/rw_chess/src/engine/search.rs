use crate::{
    engine::{
        eval::evaluate,
        movegen::pseudo_legal_moves,
    },
    rules::validation::{apply_move_to_board, is_in_check, legal_moves, CastlingRights},
    state::{
        board::Board,
        piece::{Color, Move, PieceKind},
    },
};

const INF: i32 = 1_000_000;

/// Find the best move for the given color using alpha-beta minimax search.
/// Returns `None` if there are no legal moves (checkmate/stalemate).
pub fn best_move(
    board: &Board,
    color: Color,
    depth: u32,
    en_passant: Option<crate::state::piece::Pos>,
    castling: CastlingRights,
) -> Option<Move> {
    let moves = legal_moves(board, color, en_passant, castling);
    if moves.is_empty() {
        return None;
    }

    let ordered = order_moves(board, moves);
    let maximizing = color == Color::White;

    let mut best: Option<Move> = None;
    let mut best_score = if maximizing { -INF } else { INF };

    for mv in ordered {
        let (new_board, new_ep) = apply_move_to_board(board, &mv);
        let new_castling = crate::rules::validation::update_castling_rights(castling, &mv);
        let score = alpha_beta(
            &new_board,
            depth - 1,
            -INF,
            INF,
            !maximizing,
            new_ep,
            new_castling,
        );
        if maximizing && score > best_score || !maximizing && score < best_score {
            best_score = score;
            best = Some(mv);
        }
    }

    best
}

fn alpha_beta(
    board: &Board,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    en_passant: Option<crate::state::piece::Pos>,
    castling: CastlingRights,
) -> i32 {
    let color = if maximizing { Color::White } else { Color::Black };
    let moves = legal_moves(board, color, en_passant, castling);

    if depth == 0 || moves.is_empty() {
        let mobility = moves.len();
        return evaluate(board, mobility, color);
    }

    let ordered = order_moves(board, moves);

    if maximizing {
        let mut value = -INF;
        for mv in ordered {
            let (new_board, new_ep) = apply_move_to_board(board, &mv);
            let new_castling = crate::rules::validation::update_castling_rights(castling, &mv);
            let child = alpha_beta(&new_board, depth - 1, alpha, beta, false, new_ep, new_castling);
            value = value.max(child);
            alpha = alpha.max(value);
            if alpha >= beta {
                break; // β-cutoff
            }
        }
        value
    } else {
        let mut value = INF;
        for mv in ordered {
            let (new_board, new_ep) = apply_move_to_board(board, &mv);
            let new_castling = crate::rules::validation::update_castling_rights(castling, &mv);
            let child = alpha_beta(&new_board, depth - 1, alpha, beta, true, new_ep, new_castling);
            value = value.min(child);
            beta = beta.min(value);
            if alpha >= beta {
                break; // α-cutoff
            }
        }
        value
    }
}

/// Order moves to improve alpha-beta pruning efficiency.
/// Priority: captures (by MVV-LVA), then quiet moves.
fn order_moves(board: &Board, mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_by_key(|mv| {
        // Captures get a negative score so they sort first (sort_by_key is ascending)
        if let Some(target) = board.get(mv.to) {
            let attacker_val = board.get(mv.from).map_or(0, |p| p.kind.value());
            -(target.kind.value() - attacker_val / 10) // MVV-LVA
        } else if mv.is_en_passant {
            -PieceKind::Pawn.value()
        } else {
            0
        }
    });
    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{board::Board, piece::{Piece, Pos}};

    #[test]
    fn finds_obvious_capture() {
        // White queen can capture black queen — should prefer that
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White)));
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black)));
        b.set(Pos::new(0, 0), Some(Piece::new(PieceKind::Queen, Color::White)));
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::Queen, Color::Black)));
        // White queen on a1, black queen on a8 — white should capture
        let mv = best_move(&b, Color::White, 2, None, CastlingRights::none());
        assert!(mv.is_some());
        let mv = mv.unwrap();
        assert_eq!(mv.to, Pos::new(0, 7), "Should capture black queen");
    }
}
