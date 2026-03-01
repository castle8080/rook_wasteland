use crate::{
    engine::movegen::pseudo_legal_moves,
    state::{
        board::Board,
        piece::{Color, Move, PieceKind, Pos},
    },
};

/// Castling rights tracker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    pub fn all() -> Self {
        Self {
            white_kingside: true,
            white_queenside: true,
            black_kingside: true,
            black_queenside: true,
        }
    }
    pub fn none() -> Self {
        Self {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }
}

/// Check if the given square is attacked by any piece of `attacker_color`.
pub fn is_square_attacked(board: &Board, sq: Pos, attacker_color: Color) -> bool {
    // Generate all pseudo-legal moves for attacker; if any land on sq, it's attacked.
    // We pass None for en_passant here (en passant doesn't create attacks on squares
    // in the typical sense — only captures matter).
    pseudo_legal_moves(board, attacker_color, None)
        .iter()
        .any(|m| m.to == sq)
}

/// Check if the king of `color` is currently in check.
pub fn is_in_check(board: &Board, color: Color) -> bool {
    if let Some(king_pos) = board.find_king(color) {
        is_square_attacked(board, king_pos, color.opposite())
    } else {
        false // Shouldn't happen in a real game
    }
}

/// Apply a move to a cloned board, returning the resulting board.
/// Does NOT validate legality — call legal_moves for that.
pub fn apply_move_to_board(board: &Board, mv: &Move) -> (Board, Option<Pos>) {
    let mut b = board.clone();
    let piece = b.get(mv.from).expect("No piece at move source");
    let mut new_ep: Option<Pos> = None;

    b.set(mv.from, None);

    if mv.is_en_passant {
        // Captured pawn is on the same rank as `from`, file of `to`
        let captured_pawn_pos = Pos::new(mv.to.file, mv.from.rank);
        b.set(captured_pawn_pos, None);
        b.set(mv.to, Some(piece));
    } else if mv.is_castling_kingside {
        b.set(mv.to, Some(piece));
        // Move rook
        let rook_from = Pos::new(7, mv.from.rank);
        let rook_to = Pos::new(5, mv.from.rank);
        let rook = b.get(rook_from).expect("Rook missing for castling");
        b.set(rook_from, None);
        b.set(rook_to, Some(rook));
    } else if mv.is_castling_queenside {
        b.set(mv.to, Some(piece));
        let rook_from = Pos::new(0, mv.from.rank);
        let rook_to = Pos::new(3, mv.from.rank);
        let rook = b.get(rook_from).expect("Rook missing for castling");
        b.set(rook_from, None);
        b.set(rook_to, Some(rook));
    } else if let Some(promo) = mv.promotion {
        use crate::state::piece::Piece;
        b.set(mv.to, Some(Piece::new(promo.to_piece_kind(), piece.color)));
    } else {
        b.set(mv.to, Some(piece));
        // Record en passant target for double pawn push
        if piece.kind == PieceKind::Pawn {
            let rank_diff = mv.to.rank as i32 - mv.from.rank as i32;
            if rank_diff.abs() == 2 {
                let ep_rank = ((mv.from.rank as i32 + mv.to.rank as i32) / 2) as u8;
                new_ep = Some(Pos::new(mv.from.file, ep_rank));
            }
        }
    }

    (b, new_ep)
}

/// Generate fully legal moves: pseudo-legal moves filtered to those that don't
/// leave the moving side's king in check.
pub fn legal_moves(
    board: &Board,
    color: Color,
    en_passant: Option<Pos>,
    castling: CastlingRights,
) -> Vec<Move> {
    let mut moves: Vec<Move> = pseudo_legal_moves(board, color, en_passant)
        .into_iter()
        .filter(|mv| {
            let (new_board, _) = apply_move_to_board(board, mv);
            !is_in_check(&new_board, color)
        })
        .collect();

    // Add castling moves
    castling_moves(board, color, castling, &mut moves);

    moves
}

fn castling_moves(
    board: &Board,
    color: Color,
    castling: CastlingRights,
    moves: &mut Vec<Move>,
) {
    let rank = match color {
        Color::White => 0u8,
        Color::Black => 7u8,
    };
    let king_pos = Pos::new(4, rank);
    let opp = color.opposite();

    // King must not currently be in check
    if is_in_check(board, color) {
        return;
    }

    let can_ks = match color {
        Color::White => castling.white_kingside,
        Color::Black => castling.black_kingside,
    };
    let can_qs = match color {
        Color::White => castling.white_queenside,
        Color::Black => castling.black_queenside,
    };

    // Kingside castling: squares f and g must be empty and not attacked
    if can_ks
        && board.get(Pos::new(5, rank)).is_none()
        && board.get(Pos::new(6, rank)).is_none()
        && !is_square_attacked(board, Pos::new(5, rank), opp)
        && !is_square_attacked(board, Pos::new(6, rank), opp)
    {
        moves.push(Move::castling_kingside(king_pos, Pos::new(6, rank)));
    }

    // Queenside castling: squares b, c, d must be empty; c and d not attacked
    if can_qs
        && board.get(Pos::new(1, rank)).is_none()
        && board.get(Pos::new(2, rank)).is_none()
        && board.get(Pos::new(3, rank)).is_none()
        && !is_square_attacked(board, Pos::new(2, rank), opp)
        && !is_square_attacked(board, Pos::new(3, rank), opp)
    {
        moves.push(Move::castling_queenside(king_pos, Pos::new(2, rank)));
    }
}

/// Update castling rights after a move.
pub fn update_castling_rights(rights: CastlingRights, mv: &Move) -> CastlingRights {
    let mut r = rights;
    // King moves revoke both rights for that color
    // Rook moves/captures revoke the relevant right
    // We check by source/destination squares
    if mv.from == Pos::new(4, 0) || mv.to == Pos::new(4, 0) {
        r.white_kingside = false;
        r.white_queenside = false;
    }
    if mv.from == Pos::new(4, 7) || mv.to == Pos::new(4, 7) {
        r.black_kingside = false;
        r.black_queenside = false;
    }
    if mv.from == Pos::new(7, 0) || mv.to == Pos::new(7, 0) {
        r.white_kingside = false;
    }
    if mv.from == Pos::new(0, 0) || mv.to == Pos::new(0, 0) {
        r.white_queenside = false;
    }
    if mv.from == Pos::new(7, 7) || mv.to == Pos::new(7, 7) {
        r.black_kingside = false;
    }
    if mv.from == Pos::new(0, 7) || mv.to == Pos::new(0, 7) {
        r.black_queenside = false;
    }
    r
}

/// Determine if the side to move is in checkmate.
pub fn is_checkmate(board: &Board, color: Color, en_passant: Option<Pos>, castling: CastlingRights) -> bool {
    is_in_check(board, color) && legal_moves(board, color, en_passant, castling).is_empty()
}

/// Determine if the side to move is in stalemate.
pub fn is_stalemate(board: &Board, color: Color, en_passant: Option<Pos>, castling: CastlingRights) -> bool {
    !is_in_check(board, color) && legal_moves(board, color, en_passant, castling).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{board::Board, piece::{Piece, PieceKind, Color, Pos}};

    #[test]
    fn starting_position_not_in_check() {
        let b = Board::starting_position();
        assert!(!is_in_check(&b, Color::White));
        assert!(!is_in_check(&b, Color::Black));
    }

    #[test]
    fn starting_position_white_has_twenty_moves() {
        let b = Board::starting_position();
        let moves = legal_moves(&b, Color::White, None, CastlingRights::all());
        assert_eq!(moves.len(), 20); // 16 pawn + 4 knight
    }

    #[test]
    fn back_rank_checkmate() {
        // Black king at h8, white rook at h1 (check on h-file),
        // white queen at g6 covering g7 and g8, white king at e1
        let mut b = Board::empty();
        b.set(Pos::new(7, 7), Some(Piece::new(PieceKind::King, Color::Black)));  // h8
        b.set(Pos::new(7, 0), Some(Piece::new(PieceKind::Rook, Color::White)));  // h1 (check)
        b.set(Pos::new(6, 5), Some(Piece::new(PieceKind::Queen, Color::White))); // g6
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White)));  // e1
        assert!(is_checkmate(&b, Color::Black, None, CastlingRights::none()));
    }

    #[test]
    fn lone_king_is_stalemate_when_cornered() {
        // Black king in corner, white pieces around but not checking
        let mut b = Board::empty();
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King, Color::Black))); // a8
        b.set(Pos::new(1, 5), Some(Piece::new(PieceKind::Queen, Color::White))); // b6
        b.set(Pos::new(2, 6), Some(Piece::new(PieceKind::King, Color::White))); // c7
        assert!(is_stalemate(&b, Color::Black, None, CastlingRights::none()));
    }
}
