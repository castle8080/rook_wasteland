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

    // ── Pin tests ────────────────────────────────────────────────────────────

    #[test]
    fn absolutely_pinned_bishop_has_no_legal_moves() {
        // White king e1, white bishop e4, black rook e8.
        // Any bishop move leaves the e-file and exposes the king.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King,   Color::White))); // e1
        b.set(Pos::new(4, 3), Some(Piece::new(PieceKind::Bishop, Color::White))); // e4
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King,   Color::Black))); // a8
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::Rook,   Color::Black))); // e8
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let bishop_moves: Vec<_> = moves.iter().filter(|m| m.from == Pos::new(4, 3)).collect();
        assert_eq!(bishop_moves.len(), 0, "Pinned bishop should have no legal moves");
    }

    #[test]
    fn absolutely_pinned_knight_has_no_legal_moves() {
        // Knights always change both file and rank, so they can never stay on a pin ray.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King,   Color::White))); // e1
        b.set(Pos::new(4, 3), Some(Piece::new(PieceKind::Knight, Color::White))); // e4
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King,   Color::Black))); // a8
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::Rook,   Color::Black))); // e8
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let knight_moves: Vec<_> = moves.iter().filter(|m| m.from == Pos::new(4, 3)).collect();
        assert_eq!(knight_moves.len(), 0, "Pinned knight should have no legal moves");
    }

    #[test]
    fn pinned_rook_can_only_move_along_pin_ray() {
        // White king e1, white rook e4, black rook e8.
        // The white rook is pinned to the e-file but CAN slide along it.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 3), Some(Piece::new(PieceKind::Rook, Color::White))); // e4
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King, Color::Black))); // a8
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::Rook, Color::Black))); // e8
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let rook_moves: Vec<_> = moves.iter().filter(|m| m.from == Pos::new(4, 3)).collect();
        // e-file moves: e2, e3 (below), e5, e6, e7, e8(capture) = 6
        assert_eq!(rook_moves.len(), 6, "Pinned rook must only move along the pin ray (e-file)");
        assert!(rook_moves.iter().all(|m| m.to.file == 4), "All rook moves must stay on the e-file");
    }

    // ── Check evasion tests ──────────────────────────────────────────────────

    #[test]
    fn in_check_unrelated_piece_cannot_move() {
        // Knight checks cannot be blocked — only captured or the king must move.
        // White bishop on h7 cannot capture the black knight on f3 (different diagonal)
        // and cannot block a knight check. So it has 0 legal moves.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King,   Color::White))); // e1
        b.set(Pos::new(7, 6), Some(Piece::new(PieceKind::Bishop, Color::White))); // h7
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King,   Color::Black))); // a8
        b.set(Pos::new(5, 2), Some(Piece::new(PieceKind::Knight, Color::Black))); // f3 (check: f3→e1)
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let bishop_moves: Vec<_> = moves.iter().filter(|m| m.from == Pos::new(7, 6)).collect();
        assert_eq!(bishop_moves.len(), 0,
            "Bishop cannot move when king is in knight-check and bishop can't capture the knight");
    }

    #[test]
    fn in_check_rook_can_interpose_or_king_can_move() {
        // White king e1, black rook e8 giving check.
        // White rook h8 can capture the checking rook along rank 8.
        // King can also escape to d1.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(7, 7), Some(Piece::new(PieceKind::Rook, Color::White))); // h8
        b.set(Pos::new(2, 7), Some(Piece::new(PieceKind::King, Color::Black))); // c8
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::Rook, Color::Black))); // e8 (check)
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        // White rook h8 travels left along rank 8: g8, f8, e8 (capture).
        assert!(moves.iter().any(|m| m.from == Pos::new(7, 7) && m.to == Pos::new(4, 7)),
            "White rook h8 should be able to capture the checking rook on e8");
        // King can escape to d1 (off e-file, not attacked)
        assert!(moves.iter().any(|m| m.from == Pos::new(4, 0) && m.to == Pos::new(3, 0)),
            "King should be able to escape to d1");
        // King must NOT move to e2 (still on e-file, still in check)
        assert!(!moves.iter().any(|m| m.from == Pos::new(4, 0) && m.to == Pos::new(4, 1)),
            "King cannot move to e2, still attacked by rook");
    }

    #[test]
    fn double_check_only_king_can_move() {
        // White king e1. Black rook e5 attacks along e-file. Black knight d3 attacks e1.
        // Double check: only the king can resolve it.
        // White rook c3 is present but cannot help (can't block both checks at once).
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King,   Color::White))); // e1
        b.set(Pos::new(2, 2), Some(Piece::new(PieceKind::Rook,   Color::White))); // c3
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King,   Color::Black))); // e8
        b.set(Pos::new(4, 4), Some(Piece::new(PieceKind::Rook,   Color::Black))); // e5 (check)
        b.set(Pos::new(3, 2), Some(Piece::new(PieceKind::Knight, Color::Black))); // d3 (check: d3→e1)
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        assert!(!moves.is_empty(), "King must have at least one escape in double check");
        assert!(
            moves.iter().all(|m| m.from == Pos::new(4, 0)),
            "In double check only king moves are legal; found non-king move: {:?}",
            moves.iter().find(|m| m.from != Pos::new(4, 0))
        );
    }

    // ── King safety ──────────────────────────────────────────────────────────

    #[test]
    fn king_cannot_move_into_attacked_square() {
        // Black rook d8 attacks the entire d-file. King on e1 must not be able to go to d1.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        b.set(Pos::new(3, 7), Some(Piece::new(PieceKind::Rook, Color::Black))); // d8
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        assert!(
            !moves.iter().any(|m| m.from == Pos::new(4, 0) && m.to == Pos::new(3, 0)),
            "King must not be able to move to d1 which is attacked by rook on d8"
        );
    }

    // ── En passant legality ──────────────────────────────────────────────────

    #[test]
    fn en_passant_is_legal_in_normal_position() {
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        b.set(Pos::new(4, 4), Some(Piece::new(PieceKind::Pawn, Color::White))); // e5
        b.set(Pos::new(3, 4), Some(Piece::new(PieceKind::Pawn, Color::Black))); // d5
        let ep = Some(Pos::new(3, 5)); // d6
        let moves = legal_moves(&b, Color::White, ep, CastlingRights::none());
        assert!(
            moves.iter().any(|m| m.is_en_passant && m.to == Pos::new(3, 5)),
            "En passant capture to d6 should be legal"
        );
    }

    #[test]
    fn en_passant_illegal_when_exposes_king_on_rank() {
        // The "en passant rank pin": white king a5, white pawn e5, black pawn d5 (ep target d6),
        // black rook h5. After exd6 both pawns leave rank 5, exposing the king to the rook.
        let mut b = Board::empty();
        b.set(Pos::new(0, 4), Some(Piece::new(PieceKind::King, Color::White))); // a5
        b.set(Pos::new(4, 4), Some(Piece::new(PieceKind::Pawn, Color::White))); // e5
        b.set(Pos::new(3, 4), Some(Piece::new(PieceKind::Pawn, Color::Black))); // d5
        b.set(Pos::new(7, 4), Some(Piece::new(PieceKind::Rook, Color::Black))); // h5
        b.set(Pos::new(7, 7), Some(Piece::new(PieceKind::King, Color::Black))); // h8
        let ep = Some(Pos::new(3, 5)); // d6
        let moves = legal_moves(&b, Color::White, ep, CastlingRights::none());
        assert!(
            !moves.iter().any(|m| m.is_en_passant),
            "En passant must be illegal when it exposes the king along the rank"
        );
        // The single pawn push to e6 must still be legal (black pawn on d5 still shields king)
        assert!(
            moves.iter().any(|m| m.from == Pos::new(4, 4) && m.to == Pos::new(4, 5)),
            "Single pawn push e5→e6 should remain legal"
        );
    }

    // ── Castling ─────────────────────────────────────────────────────────────

    #[test]
    fn castling_kingside_legal_when_path_clear() {
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(7, 0), Some(Piece::new(PieceKind::Rook, Color::White))); // h1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        let mut cr = CastlingRights::none();
        cr.white_kingside = true;
        let moves = legal_moves(&b, Color::White, None, cr);
        assert!(moves.iter().any(|m| m.is_castling_kingside), "Kingside castling should be legal");
    }

    #[test]
    fn castling_queenside_legal_when_path_clear() {
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(0, 0), Some(Piece::new(PieceKind::Rook, Color::White))); // a1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        let mut cr = CastlingRights::none();
        cr.white_queenside = true;
        let moves = legal_moves(&b, Color::White, None, cr);
        assert!(moves.iter().any(|m| m.is_castling_queenside), "Queenside castling should be legal");
    }

    #[test]
    fn black_castling_kingside_legal() {
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        b.set(Pos::new(7, 7), Some(Piece::new(PieceKind::Rook, Color::Black))); // h8
        let mut cr = CastlingRights::none();
        cr.black_kingside = true;
        let moves = legal_moves(&b, Color::Black, None, cr);
        assert!(moves.iter().any(|m| m.is_castling_kingside), "Black kingside castling should be legal");
    }

    #[test]
    fn cannot_castle_kingside_through_attacked_square() {
        // Black rook f8 attacks f1. King cannot pass through f1 to castle kingside.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(7, 0), Some(Piece::new(PieceKind::Rook, Color::White))); // h1
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King, Color::Black))); // a8
        b.set(Pos::new(5, 7), Some(Piece::new(PieceKind::Rook, Color::Black))); // f8 → attacks f1
        let mut cr = CastlingRights::none();
        cr.white_kingside = true;
        let moves = legal_moves(&b, Color::White, None, cr);
        assert!(!moves.iter().any(|m| m.is_castling_kingside),
            "Cannot castle kingside when f1 is attacked");
    }

    #[test]
    fn cannot_castle_when_in_check() {
        // King is in check from e8 rook. Castling rights present, but cannot castle.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(7, 0), Some(Piece::new(PieceKind::Rook, Color::White))); // h1
        b.set(Pos::new(0, 7), Some(Piece::new(PieceKind::King, Color::Black))); // a8
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::Rook, Color::Black))); // e8 (check)
        let mut cr = CastlingRights::none();
        cr.white_kingside = true;
        let moves = legal_moves(&b, Color::White, None, cr);
        assert!(!moves.iter().any(|m| m.is_castling_kingside),
            "Cannot castle when in check");
    }

    #[test]
    fn cannot_castle_when_piece_between_king_and_rook() {
        // Knight f1 blocks the kingside castling path.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King,   Color::White))); // e1
        b.set(Pos::new(7, 0), Some(Piece::new(PieceKind::Rook,   Color::White))); // h1
        b.set(Pos::new(5, 0), Some(Piece::new(PieceKind::Knight, Color::White))); // f1 blocking
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King,   Color::Black))); // e8
        let mut cr = CastlingRights::none();
        cr.white_kingside = true;
        let moves = legal_moves(&b, Color::White, None, cr);
        assert!(!moves.iter().any(|m| m.is_castling_kingside),
            "Cannot castle when piece occupies f1");
    }

    // ── Pawn capture rules ───────────────────────────────────────────────────

    #[test]
    fn pawn_cannot_capture_piece_directly_in_front() {
        // Enemy piece directly in front of pawn: push blocked and cannot "capture" forward.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        b.set(Pos::new(3, 3), Some(Piece::new(PieceKind::Pawn, Color::White))); // d4
        b.set(Pos::new(3, 4), Some(Piece::new(PieceKind::Pawn, Color::Black))); // d5 directly ahead
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let pawn_moves: Vec<_> = moves.iter().filter(|m| m.from == Pos::new(3, 3)).collect();
        assert_eq!(pawn_moves.len(), 0, "Pawn blocked by enemy ahead has no legal moves");
    }

    #[test]
    fn pawn_can_capture_both_diagonals() {
        // White pawn d4 with enemy pawns on c5 and e5.
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        b.set(Pos::new(3, 3), Some(Piece::new(PieceKind::Pawn, Color::White))); // d4
        b.set(Pos::new(2, 4), Some(Piece::new(PieceKind::Pawn, Color::Black))); // c5
        b.set(Pos::new(4, 4), Some(Piece::new(PieceKind::Pawn, Color::Black))); // e5
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let pawn_moves: Vec<_> = moves.iter().filter(|m| m.from == Pos::new(3, 3)).collect();
        // push d5 + capture c5 + capture e5 = 3
        assert_eq!(pawn_moves.len(), 3, "Pawn should push + capture left + capture right");
    }

    // ── Promotion ────────────────────────────────────────────────────────────

    #[test]
    fn promotion_legal_moves_generate_all_four_pieces() {
        let mut b = Board::empty();
        b.set(Pos::new(4, 0), Some(Piece::new(PieceKind::King, Color::White))); // e1
        b.set(Pos::new(4, 7), Some(Piece::new(PieceKind::King, Color::Black))); // e8
        b.set(Pos::new(0, 6), Some(Piece::new(PieceKind::Pawn, Color::White))); // a7
        let moves = legal_moves(&b, Color::White, None, CastlingRights::none());
        let promo_moves: Vec<_> = moves.iter().filter(|m| m.promotion.is_some()).collect();
        assert_eq!(promo_moves.len(), 4, "Should generate Q, R, B, N promotion moves");
    }

    // ── Perft ────────────────────────────────────────────────────────────────

    #[test]
    fn perft_depth_2_from_starting_position_is_400() {
        // Each of white's 20 first moves leads to 20 black responses: 20×20 = 400.
        // This is a well-known perft value and serves as a strong regression test
        // for the correctness of the full move generation pipeline.
        let b = Board::starting_position();
        let white_moves = legal_moves(&b, Color::White, None, CastlingRights::all());
        let total: usize = white_moves.iter().map(|mv| {
            let (nb, nep) = apply_move_to_board(&b, mv);
            let nc = update_castling_rights(CastlingRights::all(), mv);
            legal_moves(&nb, Color::Black, nep, nc).len()
        }).sum();
        assert_eq!(total, 400, "Perft(2) from starting position must be 400");
    }
}
