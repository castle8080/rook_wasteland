/// Chess piece color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn sign(self) -> i32 {
        match self {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

/// The type/role of a chess piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PieceKind {
    /// Base material value in centipawns.
    pub fn value(self) -> i32 {
        match self {
            PieceKind::Pawn => 100,
            PieceKind::Knight => 320,
            PieceKind::Bishop => 330,
            PieceKind::Rook => 500,
            PieceKind::Queen => 900,
            PieceKind::King => 20_000,
        }
    }
}

/// A chess piece: kind + color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

impl Piece {
    pub fn new(kind: PieceKind, color: Color) -> Self {
        Self { kind, color }
    }

    /// Unicode glyph for display.
    pub fn glyph(self) -> &'static str {
        match (self.color, self.kind) {
            (Color::White, PieceKind::King) => "♔",
            (Color::White, PieceKind::Queen) => "♕",
            (Color::White, PieceKind::Rook) => "♖",
            (Color::White, PieceKind::Bishop) => "♗",
            (Color::White, PieceKind::Knight) => "♘",
            (Color::White, PieceKind::Pawn) => "♙",
            (Color::Black, PieceKind::King) => "♚",
            (Color::Black, PieceKind::Queen) => "♛",
            (Color::Black, PieceKind::Rook) => "♜",
            (Color::Black, PieceKind::Bishop) => "♝",
            (Color::Black, PieceKind::Knight) => "♞",
            (Color::Black, PieceKind::Pawn) => "♟",
        }
    }
}

/// A board position: file (0–7, a–h) and rank (0–7, 1–8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Pos {
    pub file: u8, // 0 = a, 7 = h
    pub rank: u8, // 0 = rank 1, 7 = rank 8
}

impl Pos {
    pub fn new(file: u8, rank: u8) -> Self {
        debug_assert!(file < 8 && rank < 8, "Position out of bounds");
        Self { file, rank }
    }

    /// Convert from flat board index (0–63).
    pub fn from_index(idx: usize) -> Self {
        Self {
            file: (idx % 8) as u8,
            rank: (idx / 8) as u8,
        }
    }

    /// Convert to flat board index.
    pub fn to_index(self) -> usize {
        self.rank as usize * 8 + self.file as usize
    }

    pub fn in_bounds(file: i32, rank: i32) -> bool {
        (0..8).contains(&file) && (0..8).contains(&rank)
    }

    /// Algebraic notation string, e.g. "e4".
    pub fn to_algebraic(self) -> String {
        let file_char = (b'a' + self.file) as char;
        let rank_char = (b'1' + self.rank) as char;
        format!("{}{}", file_char, rank_char)
    }
}

/// Promotion choice for pawn promotion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Promotion {
    Queen,
    Rook,
    Bishop,
    Knight,
}

impl Promotion {
    pub fn to_piece_kind(self) -> PieceKind {
        match self {
            Promotion::Queen => PieceKind::Queen,
            Promotion::Rook => PieceKind::Rook,
            Promotion::Bishop => PieceKind::Bishop,
            Promotion::Knight => PieceKind::Knight,
        }
    }
}

/// A chess move.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: Pos,
    pub to: Pos,
    pub promotion: Option<Promotion>,
    pub is_castling_kingside: bool,
    pub is_castling_queenside: bool,
    pub is_en_passant: bool,
}

impl Move {
    pub fn new(from: Pos, to: Pos) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling_kingside: false,
            is_castling_queenside: false,
            is_en_passant: false,
        }
    }

    pub fn with_promotion(mut self, promo: Promotion) -> Self {
        self.promotion = Some(promo);
        self
    }

    pub fn castling_kingside(from: Pos, to: Pos) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling_kingside: true,
            is_castling_queenside: false,
            is_en_passant: false,
        }
    }

    pub fn castling_queenside(from: Pos, to: Pos) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling_kingside: false,
            is_castling_queenside: true,
            is_en_passant: false,
        }
    }

    pub fn en_passant(from: Pos, to: Pos) -> Self {
        Self {
            from,
            to,
            promotion: None,
            is_castling_kingside: false,
            is_castling_queenside: false,
            is_en_passant: true,
        }
    }

    /// Simple algebraic notation (not full SAN).
    pub fn to_notation(self) -> String {
        let mut s = format!("{}{}", self.from.to_algebraic(), self.to.to_algebraic());
        if let Some(promo) = self.promotion {
            let c = match promo {
                Promotion::Queen => 'q',
                Promotion::Rook => 'r',
                Promotion::Bishop => 'b',
                Promotion::Knight => 'n',
            };
            s.push(c);
        }
        s
    }
}

/// Phase of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Setup,
    Playing,
    Check,
    Checkmate,
    Stalemate,
    DrawFiftyMove,
}

/// Difficulty level for the AI engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn search_depth(self) -> u32 {
        match self {
            Difficulty::Easy => 2,
            Difficulty::Medium => 4,
            Difficulty::Hard => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_opposite() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn pos_index_roundtrip() {
        for i in 0..64usize {
            assert_eq!(Pos::from_index(i).to_index(), i);
        }
    }

    #[test]
    fn pos_algebraic() {
        assert_eq!(Pos::new(0, 0).to_algebraic(), "a1");
        assert_eq!(Pos::new(4, 3).to_algebraic(), "e4");
        assert_eq!(Pos::new(7, 7).to_algebraic(), "h8");
    }

    #[test]
    fn piece_values() {
        assert_eq!(PieceKind::Pawn.value(), 100);
        assert_eq!(PieceKind::Queen.value(), 900);
        assert_eq!(PieceKind::King.value(), 20_000);
    }

    #[test]
    fn move_notation() {
        let m = Move::new(Pos::new(4, 1), Pos::new(4, 3));
        assert_eq!(m.to_notation(), "e2e4");
    }
}
