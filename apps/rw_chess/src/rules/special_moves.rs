// Special move helpers — castling and en passant logic lives in validation.rs.
// This module provides any supplemental helpers.

use crate::state::piece::{Move, Pos};

/// Returns the position of the pawn captured by an en passant move.
pub fn en_passant_captured_pos(mv: &Move) -> Pos {
    Pos::new(mv.to.file, mv.from.rank)
}
