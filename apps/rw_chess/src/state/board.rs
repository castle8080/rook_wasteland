use crate::state::piece::{Color, Piece, PieceKind, Pos};

/// 8×8 chess board stored as flat array of 64 squares.
/// Index = rank * 8 + file  (rank 0 = rank 1, file 0 = a).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board {
    squares: [Option<Piece>; 64],
}

impl Board {
    /// Empty board.
    pub fn empty() -> Self {
        Self {
            squares: [None; 64],
        }
    }

    /// Standard chess starting position.
    pub fn starting_position() -> Self {
        let mut b = Self::empty();

        let back_rank: [PieceKind; 8] = [
            PieceKind::Rook,
            PieceKind::Knight,
            PieceKind::Bishop,
            PieceKind::Queen,
            PieceKind::King,
            PieceKind::Bishop,
            PieceKind::Knight,
            PieceKind::Rook,
        ];

        for (file, &kind) in back_rank.iter().enumerate() {
            b.set(Pos::new(file as u8, 0), Some(Piece::new(kind, Color::White)));
            b.set(Pos::new(file as u8, 1), Some(Piece::new(PieceKind::Pawn, Color::White)));
            b.set(Pos::new(file as u8, 6), Some(Piece::new(PieceKind::Pawn, Color::Black)));
            b.set(Pos::new(file as u8, 7), Some(Piece::new(kind, Color::Black)));
        }

        b
    }

    pub fn get(&self, pos: Pos) -> Option<Piece> {
        self.squares[pos.to_index()]
    }

    pub fn set(&mut self, pos: Pos, piece: Option<Piece>) {
        self.squares[pos.to_index()] = piece;
    }

    /// Find position of king for the given color.
    pub fn find_king(&self, color: Color) -> Option<Pos> {
        self.squares.iter().enumerate().find_map(|(i, sq)| {
            if let Some(p) = sq
                && p.kind == PieceKind::King && p.color == color {
                    return Some(Pos::from_index(i));
                }
            None
        })
    }

    /// All positions with pieces of the given color.
    pub fn pieces_of(&self, color: Color) -> Vec<(Pos, Piece)> {
        self.squares
            .iter()
            .enumerate()
            .filter_map(|(i, sq)| {
                sq.filter(|p| p.color == color)
                    .map(|p| (Pos::from_index(i), p))
            })
            .collect()
    }

    /// Count material for scoring (positive = white advantage).
    pub fn material_balance(&self) -> i32 {
        self.squares.iter().fold(0, |acc, sq| {
            acc + sq.map_or(0, |p| p.kind.value() * p.color.sign())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_position_piece_count() {
        let b = Board::starting_position();
        let white = b.pieces_of(Color::White);
        let black = b.pieces_of(Color::Black);
        assert_eq!(white.len(), 16);
        assert_eq!(black.len(), 16);
    }

    #[test]
    fn find_king_starting() {
        let b = Board::starting_position();
        assert_eq!(b.find_king(Color::White), Some(Pos::new(4, 0))); // e1
        assert_eq!(b.find_king(Color::Black), Some(Pos::new(4, 7))); // e8
    }

    #[test]
    fn material_balance_starting() {
        // Starting position is symmetric → balance = 0
        let b = Board::starting_position();
        assert_eq!(b.material_balance(), 0);
    }
}
