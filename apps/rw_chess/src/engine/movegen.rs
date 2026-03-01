use crate::state::{
    board::Board,
    piece::{Color, Move, Piece, PieceKind, Pos, Promotion},
};

/// Generate all pseudo-legal moves for the given color.
/// "Pseudo-legal" means geometrically valid but may leave the king in check.
/// Call `legal_moves` from the rules layer for fully validated moves.
pub fn pseudo_legal_moves(board: &Board, color: Color, en_passant: Option<Pos>) -> Vec<Move> {
    let mut moves = Vec::new();
    for (pos, piece) in board.pieces_of(color) {
        match piece.kind {
            PieceKind::Pawn => pawn_moves(board, pos, piece, en_passant, &mut moves),
            PieceKind::Knight => knight_moves(board, pos, piece, &mut moves),
            PieceKind::Bishop => sliding_moves(board, pos, piece, &BISHOP_DIRS, &mut moves),
            PieceKind::Rook => sliding_moves(board, pos, piece, &ROOK_DIRS, &mut moves),
            PieceKind::Queen => sliding_moves(board, pos, piece, &QUEEN_DIRS, &mut moves),
            PieceKind::King => king_moves(board, pos, piece, &mut moves),
        }
    }
    moves
}

const BISHOP_DIRS: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
const ROOK_DIRS: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const QUEEN_DIRS: [(i32, i32); 8] = [
    (-1, -1), (-1, 0), (-1, 1),
    (0, -1),           (0, 1),
    (1, -1),  (1, 0),  (1, 1),
];

fn sliding_moves(
    board: &Board,
    from: Pos,
    piece: Piece,
    dirs: &[(i32, i32)],
    moves: &mut Vec<Move>,
) {
    for &(df, dr) in dirs {
        let mut f = from.file as i32 + df;
        let mut r = from.rank as i32 + dr;
        while Pos::in_bounds(f, r) {
            let to = Pos::new(f as u8, r as u8);
            match board.get(to) {
                None => moves.push(Move::new(from, to)),
                Some(target) => {
                    if target.color != piece.color {
                        moves.push(Move::new(from, to)); // capture
                    }
                    break; // blocked
                }
            }
            f += df;
            r += dr;
        }
    }
}

fn knight_moves(board: &Board, from: Pos, piece: Piece, moves: &mut Vec<Move>) {
    const OFFSETS: [(i32, i32); 8] = [
        (-2, -1), (-2, 1), (-1, -2), (-1, 2),
        (1, -2),  (1, 2),  (2, -1),  (2, 1),
    ];
    for &(df, dr) in &OFFSETS {
        let f = from.file as i32 + df;
        let r = from.rank as i32 + dr;
        if Pos::in_bounds(f, r) {
            let to = Pos::new(f as u8, r as u8);
            match board.get(to) {
                None => moves.push(Move::new(from, to)),
                Some(target) if target.color != piece.color => moves.push(Move::new(from, to)),
                _ => {}
            }
        }
    }
}

fn king_moves(board: &Board, from: Pos, piece: Piece, moves: &mut Vec<Move>) {
    for &(df, dr) in &QUEEN_DIRS {
        let f = from.file as i32 + df;
        let r = from.rank as i32 + dr;
        if Pos::in_bounds(f, r) {
            let to = Pos::new(f as u8, r as u8);
            match board.get(to) {
                None => moves.push(Move::new(from, to)),
                Some(target) if target.color != piece.color => moves.push(Move::new(from, to)),
                _ => {}
            }
        }
    }
}

fn pawn_moves(
    board: &Board,
    from: Pos,
    piece: Piece,
    en_passant: Option<Pos>,
    moves: &mut Vec<Move>,
) {
    let (dir, start_rank, promo_rank): (i32, u8, u8) = match piece.color {
        Color::White => (1, 1, 7),
        Color::Black => (-1, 6, 0),
    };

    let rank = from.rank as i32;
    let file = from.file as i32;

    // Single push
    let nr = rank + dir;
    if Pos::in_bounds(file, nr) {
        let to = Pos::new(file as u8, nr as u8);
        if board.get(to).is_none() {
            push_pawn_moves(from, to, to.rank == promo_rank, moves);

            // Double push from starting rank
            if from.rank == start_rank {
                let nr2 = rank + dir * 2;
                let to2 = Pos::new(file as u8, nr2 as u8);
                if board.get(to2).is_none() {
                    moves.push(Move::new(from, to2));
                }
            }
        }
    }

    // Captures
    for &df in &[-1i32, 1] {
        let nf = file + df;
        if Pos::in_bounds(nf, nr) {
            let to = Pos::new(nf as u8, nr as u8);
            // Normal capture
            if let Some(target) = board.get(to) {
                if target.color != piece.color {
                    push_pawn_moves(from, to, to.rank == promo_rank, moves);
                }
            }
            // En passant
            if Some(to) == en_passant {
                moves.push(Move::en_passant(from, to));
            }
        }
    }
}

fn push_pawn_moves(from: Pos, to: Pos, is_promotion: bool, moves: &mut Vec<Move>) {
    if is_promotion {
        for promo in [Promotion::Queen, Promotion::Rook, Promotion::Bishop, Promotion::Knight] {
            moves.push(Move::new(from, to).with_promotion(promo));
        }
    } else {
        moves.push(Move::new(from, to));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knight_center_has_eight_moves() {
        let mut board = Board::empty();
        let pos = Pos::new(3, 3); // d4
        board.set(pos, Some(Piece::new(PieceKind::Knight, Color::White)));
        let moves = pseudo_legal_moves(&board, Color::White, None);
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn pawn_starting_has_two_pushes() {
        let mut board = Board::empty();
        let pos = Pos::new(4, 1); // e2
        board.set(pos, Some(Piece::new(PieceKind::Pawn, Color::White)));
        let moves = pseudo_legal_moves(&board, Color::White, None);
        assert_eq!(moves.len(), 2); // e3 and e4
    }

    #[test]
    fn rook_open_file_has_fourteen_moves() {
        let mut board = Board::empty();
        let pos = Pos::new(0, 0); // a1
        board.set(pos, Some(Piece::new(PieceKind::Rook, Color::White)));
        let moves = pseudo_legal_moves(&board, Color::White, None);
        assert_eq!(moves.len(), 14); // 7 along file + 7 along rank
    }

    #[test]
    fn bishop_corner_has_seven_moves() {
        let mut board = Board::empty();
        board.set(Pos::new(0, 0), Some(Piece::new(PieceKind::Bishop, Color::White)));
        let moves = pseudo_legal_moves(&board, Color::White, None);
        assert_eq!(moves.len(), 7);
    }

    #[test]
    fn pawn_promotion_generates_four_moves() {
        let mut board = Board::empty();
        board.set(Pos::new(0, 6), Some(Piece::new(PieceKind::Pawn, Color::White)));
        let moves = pseudo_legal_moves(&board, Color::White, None);
        assert_eq!(moves.len(), 4); // Q, R, B, N
    }
}
