use crate::state::{
    board::Board,
    piece::{Color, PieceKind, Pos},
};

/// Evaluate the board from White's perspective (positive = White advantage).
/// Score is in centipawns.
pub fn evaluate(board: &Board, legal_move_count: usize, color_to_move: Color) -> i32 {
    let material = board.material_balance();
    let phase = game_phase(board);
    let positional = positional_score(board, phase);
    let mobility = mobility_score(legal_move_count, color_to_move);
    let pawn_struct = pawn_structure_score(board);
    let king_safe = king_safety_score(board, phase);
    let rook_bonus = rook_file_score(board);
    let bishop_bonus = bishop_pair_score(board);
    material + positional + mobility + pawn_struct + king_safe + rook_bonus + bishop_bonus
}

/// Game phase: 256 = full middlegame, 0 = full endgame.
/// Computed from non-pawn material on board (Queen=4, Rook=2, minor=1, max=24).
fn game_phase(board: &Board) -> i32 {
    let mut phase = 0i32;
    for rank in 0u8..8 {
        for file in 0u8..8 {
            if let Some(p) = board.get(Pos::new(file, rank)) {
                phase += match p.kind {
                    PieceKind::Queen => 4,
                    PieceKind::Rook => 2,
                    PieceKind::Knight | PieceKind::Bishop => 1,
                    _ => 0,
                };
            }
        }
    }
    // Clamp to [0, 24] and scale to [0, 256].
    phase.min(24) * 256 / 24
}

fn positional_score(board: &Board, phase: i32) -> i32 {
    let mut score = 0i32;
    for rank in 0u8..8 {
        for file in 0u8..8 {
            let pos = Pos::new(file, rank);
            if let Some(piece) = board.get(pos) {
                let table_val = piece_square_value(piece.kind, piece.color, pos, phase);
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
/// King table interpolates between middlegame and endgame based on phase.
fn piece_square_value(kind: PieceKind, color: Color, pos: Pos, phase: i32) -> i32 {
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
        // Interpolate: middlegame king wants safety, endgame king wants activity.
        PieceKind::King => {
            let mg = KING_MIDDLE_TABLE[idx];
            let eg = KING_END_TABLE[idx];
            (mg * phase + eg * (256 - phase)) / 256
        }
    }
}

/// Pawn structure evaluation: doubled, isolated, and passed pawns.
fn pawn_structure_score(board: &Board) -> i32 {
    let mut score = 0i32;

    // Per-file pawn counts and most-advanced rank per color.
    // White advances toward rank 7: track max rank.
    // Black advances toward rank 0: track min rank.
    let mut white_count = [0u8; 8];
    let mut black_count = [0u8; 8];
    let mut white_max_rank = [0u8; 8]; // most advanced white pawn rank per file
    let mut black_min_rank = [7u8; 8]; // most advanced black pawn rank per file
    // For passed-pawn blocking check, need the "farthest back" opponent pawn:
    let mut black_max_rank = [0u8; 8]; // highest rank (farthest back) black pawn per file
    let mut white_min_rank = [7u8; 8]; // lowest rank (farthest back) white pawn per file

    for rank in 0u8..8 {
        for file in 0u8..8 {
            if let Some(p) = board.get(Pos::new(file, rank)) {
                if p.kind != PieceKind::Pawn {
                    continue;
                }
                let f = file as usize;
                match p.color {
                    Color::White => {
                        white_count[f] += 1;
                        if rank > white_max_rank[f] {
                            white_max_rank[f] = rank;
                        }
                        if rank < white_min_rank[f] {
                            white_min_rank[f] = rank;
                        }
                    }
                    Color::Black => {
                        black_count[f] += 1;
                        if rank < black_min_rank[f] {
                            black_min_rank[f] = rank;
                        }
                        if rank > black_max_rank[f] {
                            black_max_rank[f] = rank;
                        }
                    }
                }
            }
        }
    }

    for f in 0..8usize {
        // --- White pawn penalties / bonuses ---
        if white_count[f] > 0 {
            // Doubled pawns
            if white_count[f] > 1 {
                score -= 15 * (white_count[f] - 1) as i32;
            }
            // Isolated pawns (no friendly pawn on adjacent file)
            let left = f > 0 && white_count[f - 1] > 0;
            let right = f < 7 && white_count[f + 1] > 0;
            if !left && !right {
                score -= 20;
            }
            // Passed pawn: no black pawn ahead on this file or adjacent files.
            // "Ahead" for white = higher rank. black_max_rank defaults to 0, so if
            // there's no black pawn on a file, the condition black_max_rank[cf] <= wr is met.
            let wr = white_max_rank[f];
            let passed = passed_for_files(f, |cf| black_max_rank[cf] <= wr);
            if passed {
                // Advancement = distance from starting rank (1). Capped at 5 (rank 6).
                let advance = wr.saturating_sub(1) as usize;
                score += PASSED_PAWN_BONUS[advance.min(5)];
            }
        }

        // --- Black pawn penalties / bonuses ---
        if black_count[f] > 0 {
            if black_count[f] > 1 {
                score += 15 * (black_count[f] - 1) as i32; // bad for black = good for white
            }
            let left = f > 0 && black_count[f - 1] > 0;
            let right = f < 7 && black_count[f + 1] > 0;
            if !left && !right {
                score += 20; // bad for black
            }
            // Passed pawn: no white pawn ahead (lower rank) on this or adjacent files.
            // white_min_rank defaults to 7 when there's no white pawn on a file.
            let br = black_min_rank[f];
            let passed = passed_for_files(f, |cf| white_min_rank[cf] >= br);
            if passed {
                let advance = 6u8.saturating_sub(br) as usize;
                score -= PASSED_PAWN_BONUS[advance.min(5)]; // good for black = negative from white's POV
            }
        }
    }

    score
}

/// Helper: checks the 2-3 files around `f` and returns true if `predicate` holds for all.
#[inline]
fn passed_for_files(f: usize, predicate: impl Fn(usize) -> bool) -> bool {
    let lo = f.saturating_sub(1);
    let hi = (f + 1).min(7);
    (lo..=hi).all(predicate)
}

/// Passed pawn bonus indexed by advancement (0 = starting rank, 5 = one step from promotion).
const PASSED_PAWN_BONUS: [i32; 6] = [0, 10, 20, 35, 55, 75];

/// King safety: pawn shield in front of the king, only meaningful in middlegame.
fn king_safety_score(board: &Board, phase: i32) -> i32 {
    if phase < 32 {
        return 0; // pure endgame — king safety irrelevant
    }
    let mut score = 0i32;
    for color in [Color::White, Color::Black] {
        let Some(king) = board.find_king(color) else {
            continue;
        };
        let forward: i8 = if color == Color::White { 1 } else { -1 };
        let mut shield = 0i32;
        for df in -1i8..=1i8 {
            let cf = king.file as i8 + df;
            if !(0..8).contains(&cf) {
                continue;
            }
            for steps in 1i8..=2i8 {
                let rf = king.rank as i8 + forward * steps;
                if !(0..8).contains(&rf) {
                    continue;
                }
                if let Some(p) = board.get(Pos::new(cf as u8, rf as u8))
                    && p.kind == PieceKind::Pawn && p.color == color
                {
                    shield += if steps == 1 { 15 } else { 8 };
                }
            }
        }
        // Scale by phase: full bonus in middlegame, zero in endgame.
        score += color.sign() * shield * phase / 256;
    }
    score
}

/// Rook bonuses: open file (+25) and semi-open file (+12).
fn rook_file_score(board: &Board) -> i32 {
    let mut score = 0i32;
    for rank in 0u8..8 {
        for file in 0u8..8 {
            let pos = Pos::new(file, rank);
            let Some(piece) = board.get(pos) else {
                continue;
            };
            if piece.kind != PieceKind::Rook {
                continue;
            }
            let (has_friendly, has_enemy) = file_pawn_counts(board, file, piece.color);
            let bonus = if !has_friendly && !has_enemy {
                25 // open file
            } else if !has_friendly {
                12 // semi-open file
            } else {
                0
            };
            score += piece.color.sign() * bonus;
        }
    }
    score
}

fn file_pawn_counts(board: &Board, file: u8, color: Color) -> (bool, bool) {
    let mut has_friendly = false;
    let mut has_enemy = false;
    for rank in 0u8..8 {
        if let Some(p) = board.get(Pos::new(file, rank)) && p.kind == PieceKind::Pawn {
                if p.color == color {
                    has_friendly = true;
                } else {
                    has_enemy = true;
                }
            }
    }
    (has_friendly, has_enemy)
}

/// Bishop pair bonus: +50cp if a side has two or more bishops.
fn bishop_pair_score(board: &Board) -> i32 {
    let mut white_bishops = 0i32;
    let mut black_bishops = 0i32;
    for rank in 0u8..8 {
        for file in 0u8..8 {
            if let Some(p) = board.get(Pos::new(file, rank)) && p.kind == PieceKind::Bishop {
                    match p.color {
                        Color::White => white_bishops += 1,
                        Color::Black => black_bishops += 1,
                    }
                }
        }
    }
    let white_bonus = if white_bishops >= 2 { 50 } else { 0 };
    let black_bonus = if black_bishops >= 2 { 50 } else { 0 };
    white_bonus - black_bonus
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

/// King endgame table: king should centralize and become active.
#[rustfmt::skip]
const KING_END_TABLE: [i32; 64] = [
    -50,-40,-30,-20,-20,-30,-40,-50,
    -30,-20,-10,  0,  0,-10,-20,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 30, 40, 40, 30,-10,-30,
    -30,-10, 20, 30, 30, 20,-10,-30,
    -30,-30,  0,  0,  0,  0,-30,-30,
    -50,-30,-30,-30,-30,-30,-30,-50,
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

    #[test]
    fn passed_pawn_bonus_applied() {
        // White pawn far advanced with no black pawns — should score higher than starting.
        let mut b = Board::starting_position();
        // Remove all pawns, place a lone white pawn on rank 5 (advanced)
        for f in 0..8u8 {
            b.set(Pos::new(f, 1), None); // white pawns
            b.set(Pos::new(f, 6), None); // black pawns
        }
        b.set(Pos::new(3, 5), Some(crate::state::piece::Piece::new(PieceKind::Pawn, Color::White)));
        let score = evaluate(&b, 20, Color::White);
        assert!(score > 0, "White with advanced passed pawn should score positively: {score}");
    }

    #[test]
    fn doubled_pawn_penalty_applied() {
        // Two pawns spread across adjacent files (no isolation, no doubling) should score
        // better than two pawns stacked on one file (doubled + isolated).
        let eval_with_setup = |doubled: bool| {
            let mut b = Board::empty();
            b.set(Pos::new(4, 0), Some(crate::state::piece::Piece::new(PieceKind::King, Color::White)));
            b.set(Pos::new(4, 7), Some(crate::state::piece::Piece::new(PieceKind::King, Color::Black)));
            if doubled {
                // Stack two pawns on file 3 (doubled + isolated on that file)
                b.set(Pos::new(3, 2), Some(crate::state::piece::Piece::new(PieceKind::Pawn, Color::White)));
                b.set(Pos::new(3, 3), Some(crate::state::piece::Piece::new(PieceKind::Pawn, Color::White)));
            } else {
                // Two pawns on adjacent files at same ranks — not doubled, not isolated
                b.set(Pos::new(3, 3), Some(crate::state::piece::Piece::new(PieceKind::Pawn, Color::White)));
                b.set(Pos::new(4, 3), Some(crate::state::piece::Piece::new(PieceKind::Pawn, Color::White)));
            }
            evaluate(&b, 20, Color::White)
        };
        let doubled = eval_with_setup(true);
        let normal = eval_with_setup(false);
        assert!(doubled < normal, "Doubled + isolated pawns should score lower: doubled={doubled} normal={normal}");
    }

    #[test]
    fn bishop_pair_bonus_applied() {
        let mut b = Board::starting_position();
        // Remove black's bishops to give white bishop pair advantage
        b.set(Pos::new(2, 7), None); // black bishop c8
        b.set(Pos::new(5, 7), None); // black bishop f8
        let score = evaluate(&b, 20, Color::White);
        assert!(score > 600, "White with bishop pair advantage should score well: {score}");
    }
}
