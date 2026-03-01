use crate::state::{
    board::Board,
    piece::{Color, PieceKind, Pos},
};

/// Evaluate the board from White's perspective (positive = White advantage).
/// Score is in centipawns.
pub fn evaluate(board: &Board, legal_move_count: usize, color_to_move: Color) -> i32 {
    let material = board.material_balance();
    let positional = positional_score(board);
    let mobility = mobility_score(legal_move_count, color_to_move);
    material + positional + mobility
}

fn positional_score(board: &Board) -> i32 {
    let mut score = 0i32;
    for rank in 0u8..8 {
        for file in 0u8..8 {
            let pos = Pos::new(file, rank);
            if let Some(piece) = board.get(pos) {
                let table_val = piece_square_value(piece.kind, piece.color, pos);
                score += table_val * piece.color.sign();
            }
        }
    }
    score
}

fn mobility_score(legal_move_count: usize, color: Color) -> i32 {
    (legal_move_count as i32 * 5) * color.sign()
}

/// Piece-square table lookup. Returns a bonus (positive = good for that piece's color).
fn piece_square_value(kind: PieceKind, color: Color, pos: Pos) -> i32 {
    // Tables are from White's perspective (rank 0 = White's back rank).
    // For Black, we mirror the rank.
    let rank = match color {
        Color::White => pos.rank as usize,
        Color::Black => 7 - pos.rank as usize,
    };
    let file = pos.file as usize;
    let idx = rank * 8 + file;

    match kind {
        PieceKind::Pawn => PAWN_TABLE[idx],
        PieceKind::Knight => KNIGHT_TABLE[idx],
        PieceKind::Bishop => BISHOP_TABLE[idx],
        PieceKind::Rook => ROOK_TABLE[idx],
        PieceKind::Queen => QUEEN_TABLE[idx],
        PieceKind::King => KING_MIDDLE_TABLE[idx],
    }
}

// Piece-square tables (centipawns). Stored rank 0→7 (White: back rank first).
// Values based on classical chess programming heuristics.

#[rustfmt::skip]
const PAWN_TABLE: [i32; 64] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10,-20,-20, 10, 10,  5,
     5, -5,-10,  0,  0,-10, -5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5,  5, 10, 25, 25, 10,  5,  5,
    10, 10, 20, 30, 30, 20, 10, 10,
    50, 50, 50, 50, 50, 50, 50, 50,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const KNIGHT_TABLE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const BISHOP_TABLE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const ROOK_TABLE: [i32; 64] = [
     0,  0,  0,  5,  5,  0,  0,  0,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     5, 10, 10, 10, 10, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0,
];

#[rustfmt::skip]
const QUEEN_TABLE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -10,  5,  5,  5,  5,  5,  0,-10,
      0,  0,  5,  5,  5,  5,  0, -5,
     -5,  0,  5,  5,  5,  5,  0, -5,
    -10,  0,  5,  5,  5,  5,  0,-10,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20,
];

#[rustfmt::skip]
const KING_MIDDLE_TABLE: [i32; 64] = [
     20, 30, 10,  0,  0, 10, 30, 20,
     20, 20,  0,  0,  0,  0, 20, 20,
    -10,-20,-20,-20,-20,-20,-20,-10,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::board::Board;

    #[test]
    fn starting_position_score_is_near_zero() {
        let b = Board::starting_position();
        // Material is balanced; positional should be close to 0
        let score = evaluate(&b, 20, Color::White);
        assert!(score.abs() < 200, "Starting position score too far from 0: {score}");
    }

    #[test]
    fn material_advantage_reflected() {
        let mut b = Board::starting_position();
        // Remove a black queen
        b.set(Pos::new(3, 7), None);
        let score = evaluate(&b, 20, Color::White);
        assert!(score > 800, "Should show white advantage after removing black queen: {score}");
    }
}
