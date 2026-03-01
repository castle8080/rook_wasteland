use crate::{
    engine::eval::evaluate,
    rules::validation::{apply_move_to_board, legal_moves, update_castling_rights, CastlingRights},
    state::{
        board::Board,
        piece::{Color, Move, PieceKind, Pos},
    },
};

const INF: i32 = 1_000_000;

/// Hard cap on nodes searched per call. When hit, iterative deepening returns
/// the best result from the last *fully completed* depth instead of freezing.
/// ~300k nodes ≈ <500ms in browser WASM for most positions.
const NODE_LIMIT: u32 = 300_000;

/// Beam width: max moves considered at each interior node.
/// Cuts effective branching factor from ~35 → 12. Captures and checks are
/// always ordered first by MVV-LVA, so the pruned tail is mostly quiet moves
/// that wouldn't change the result.
const BEAM_WIDTH: usize = 12;

/// Max additional plies of capture-only search appended after the main horizon.
/// Prevents horizon blunders (e.g. stopping mid-exchange).
const QUIESCENCE_DEPTH: u32 = 4;

/// Per-search mutable state threaded through alpha-beta and quiescence.
struct SearchCtx<'a> {
    en_passant: Option<Pos>,
    castling: CastlingRights,
    nodes: &'a mut u32,
}

/// Find the best move using iterative-deepening alpha-beta with beam pruning
/// and quiescence search. Returns `None` only when there are no legal moves.
pub fn best_move(
    board: &Board,
    color: Color,
    max_depth: u32,
    en_passant: Option<Pos>,
    castling: CastlingRights,
) -> Option<Move> {
    let moves = legal_moves(board, color, en_passant, castling);
    if moves.is_empty() {
        return None;
    }

    let ordered = order_moves(board, moves);
    let maximizing = color == Color::White;
    let mut nodes = 0u32;

    // Fallback: top move from ordering alone (good enough if depth 1 is also cut short)
    let mut best = ordered[0];

    for depth in 1..=max_depth {
        let mut depth_best = ordered[0];
        let mut depth_best_score = if maximizing { -INF } else { INF };
        let mut completed = true;

        for &mv in &ordered {
            let (nb, nep) = apply_move_to_board(board, &mv);
            let nc = update_castling_rights(castling, &mv);
            let mut ctx = SearchCtx { en_passant: nep, castling: nc, nodes: &mut nodes };
            let score = alpha_beta(&nb, depth - 1, -INF, INF, !maximizing, &mut ctx);

            if (maximizing && score > depth_best_score) || (!maximizing && score < depth_best_score) {
                depth_best_score = score;
                depth_best = mv;
            }

            if nodes >= NODE_LIMIT {
                completed = false;
                break;
            }
        }

        if completed {
            best = depth_best;
        } else {
            // Node limit hit mid-depth — use last fully-completed depth's result.
            break;
        }
    }

    Some(best)
}

fn alpha_beta(
    board: &Board,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    ctx: &mut SearchCtx,
) -> i32 {
    *ctx.nodes += 1;

    let color = if maximizing { Color::White } else { Color::Black };
    let moves = legal_moves(board, color, ctx.en_passant, ctx.castling);

    if depth == 0 || moves.is_empty() {
        // Hand off to quiescence instead of a hard static cutoff.
        return quiescence(board, alpha, beta, maximizing, ctx, QUIESCENCE_DEPTH);
    }

    // Beam: only search the top BEAM_WIDTH moves; the ordered tail is pruned.
    let ordered = order_moves(board, moves);
    let beam = ordered.iter().take(BEAM_WIDTH);

    if maximizing {
        let mut value = -INF;
        for mv in beam {
            let (nb, nep) = apply_move_to_board(board, mv);
            let nc = update_castling_rights(ctx.castling, mv);
            let mut child_ctx = SearchCtx { en_passant: nep, castling: nc, nodes: ctx.nodes };
            let child = alpha_beta(&nb, depth - 1, alpha, beta, false, &mut child_ctx);
            value = value.max(child);
            alpha = alpha.max(value);
            if alpha >= beta || *ctx.nodes >= NODE_LIMIT {
                break;
            }
        }
        value
    } else {
        let mut value = INF;
        for mv in beam {
            let (nb, nep) = apply_move_to_board(board, mv);
            let nc = update_castling_rights(ctx.castling, mv);
            let mut child_ctx = SearchCtx { en_passant: nep, castling: nc, nodes: ctx.nodes };
            let child = alpha_beta(&nb, depth - 1, alpha, beta, true, &mut child_ctx);
            value = value.min(child);
            beta = beta.min(value);
            if alpha >= beta || *ctx.nodes >= NODE_LIMIT {
                break;
            }
        }
        value
    }
}

/// Quiescence search: after the main horizon, keep searching captures until
/// the position is "quiet" (no captures available). This avoids evaluating a
/// position mid-exchange and dramatically reduces horizon blunders.
fn quiescence(
    board: &Board,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    ctx: &mut SearchCtx,
    depth: u32,
) -> i32 {
    *ctx.nodes += 1;

    let color = if maximizing { Color::White } else { Color::Black };
    let all_moves = legal_moves(board, color, ctx.en_passant, ctx.castling);

    // Stand-pat: the side to move can always "stand pat" (do nothing) —
    // if the static eval already beats beta, no need to search captures.
    let stand_pat = evaluate(board, all_moves.len(), color);

    if depth == 0 {
        return stand_pat;
    }

    let captures: Vec<Move> = all_moves
        .into_iter()
        .filter(|m| board.get(m.to).is_some() || m.is_en_passant)
        .collect();

    if captures.is_empty() {
        return stand_pat;
    }

    let ordered = order_moves(board, captures);

    if maximizing {
        if stand_pat >= beta {
            return beta;
        }
        alpha = alpha.max(stand_pat);

        for mv in ordered {
            let (nb, nep) = apply_move_to_board(board, &mv);
            let nc = update_castling_rights(ctx.castling, &mv);
            let mut child_ctx = SearchCtx { en_passant: nep, castling: nc, nodes: ctx.nodes };
            let score = quiescence(&nb, alpha, beta, false, &mut child_ctx, depth - 1);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        alpha
    } else {
        if stand_pat <= alpha {
            return alpha;
        }
        beta = beta.min(stand_pat);

        for mv in ordered {
            let (nb, nep) = apply_move_to_board(board, &mv);
            let nc = update_castling_rights(ctx.castling, &mv);
            let mut child_ctx = SearchCtx { en_passant: nep, castling: nc, nodes: ctx.nodes };
            let score = quiescence(&nb, alpha, beta, true, &mut child_ctx, depth - 1);
            beta = beta.min(score);
            if alpha >= beta {
                break;
            }
        }
        beta
    }
}

/// Order moves for alpha-beta efficiency and beam selection quality.
/// Captures sorted by MVV-LVA (most valuable victim, least valuable attacker first).
fn order_moves(board: &Board, mut moves: Vec<Move>) -> Vec<Move> {
    moves.sort_by_key(|mv| {
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
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White)));
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black)));
        b.set(Pos::new(0, 0), Some(Piece::new(PieceKind::Queen, Color::White)));
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::Queen, Color::Black)));
        let mv = best_move(&b, Color::White, 2, None, CastlingRights::none());
        assert!(mv.is_some());
        assert_eq!(mv.unwrap().to, Pos::new(0, 7), "Should capture black queen");
    }

    #[test]
    fn node_limit_still_returns_a_move() {
        // Even in a complex starting position, best_move must always return Some.
        let b = Board::starting_position();
        let mv = best_move(&b, Color::White, 6, None, CastlingRights::all());
        assert!(mv.is_some(), "Must return a move even at max depth");
    }

    #[test]
    fn does_not_blunder_into_free_piece() {
        // White has a free black rook on a8; engine should take it.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White)));
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black)));
        b.set(Pos::new(0, 1), Some(Piece::new(PieceKind::Rook, Color::White)));
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::Rook, Color::Black)));
        let mv = best_move(&b, Color::White, 4, None, CastlingRights::none());
        assert!(mv.is_some());
        assert_eq!(mv.unwrap().to, Pos::new(0, 7), "Should capture free rook");
    }

    #[test]
    fn quiescence_avoids_bad_capture() {
        // White queen can capture a pawn defended by a black rook — should avoid.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White)));
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black)));
        b.set(Pos::new(3, 0), Some(Piece::new(PieceKind::Queen, Color::White)));
        b.set(Pos::new(3, 4), Some(Piece::new(PieceKind::Pawn, Color::Black)));
        b.set(Pos::new(3, 7), Some(Piece::new(PieceKind::Rook, Color::Black)));
        // Queen×pawn followed by Rook×queen = -800cp for white; should not capture.
        let mv = best_move(&b, Color::White, 3, None, CastlingRights::none());
        assert!(mv.is_some());
        assert_ne!(mv.unwrap().to, Pos::new(3, 4), "Should not take defended pawn with queen");
    }
}
