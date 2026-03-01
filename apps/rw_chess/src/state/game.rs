use leptos::prelude::*;

use crate::{
    engine::persona::Persona,
    rules::validation::{
        apply_move_to_board, is_checkmate, is_in_check, is_stalemate, legal_moves,
        update_castling_rights, CastlingRights,
    },
    state::{
        board::Board,
        piece::{Color, Difficulty, GamePhase, Move, Piece, Pos},
    },
};

/// A record of a completed move for history display.
#[derive(Clone, Debug)]
pub struct MoveRecord {
    pub mv: Move,
    pub notation: String,
    pub piece: Piece,
    pub captured: Option<Piece>,
}

/// All reactive game state, provided via Leptos context.
#[derive(Clone)]
pub struct GameState {
    pub board: RwSignal<Board>,
    pub active_color: RwSignal<Color>,
    pub phase: RwSignal<GamePhase>,
    pub en_passant: RwSignal<Option<Pos>>,
    pub castling: RwSignal<CastlingRights>,
    pub move_history: RwSignal<Vec<MoveRecord>>,
    pub selected_square: RwSignal<Option<Pos>>,
    pub valid_moves_for_selected: RwSignal<Vec<Move>>,
    pub last_move: RwSignal<Option<Move>>,
    pub engine_highlight: RwSignal<Option<Move>>,
    pub player_color: RwSignal<Color>,
    pub player_name: RwSignal<String>,
    pub difficulty: RwSignal<Difficulty>,
    pub halfmove_clock: RwSignal<u32>,
    pub captured_white: RwSignal<Vec<Piece>>,
    pub captured_black: RwSignal<Vec<Piece>>,
    /// Latest commentary line from the engine persona.
    pub commentary: RwSignal<Option<String>>,
    /// Incremented each time commentary changes; drives animation re-trigger in the UI.
    pub commentary_gen: RwSignal<u32>,
    /// The engine persona for the current game (set on game start).
    pub persona: RwSignal<Persona>,
}

impl GameState {
    pub fn new() -> Self {
        use crate::engine::persona::{persona_for_difficulty};
        Self {
            board: RwSignal::new(Board::starting_position()),
            active_color: RwSignal::new(Color::White),
            phase: RwSignal::new(GamePhase::Playing),
            en_passant: RwSignal::new(None),
            castling: RwSignal::new(CastlingRights::all()),
            move_history: RwSignal::new(Vec::new()),
            selected_square: RwSignal::new(None),
            valid_moves_for_selected: RwSignal::new(Vec::new()),
            last_move: RwSignal::new(None),
            engine_highlight: RwSignal::new(None),
            player_color: RwSignal::new(Color::White),
            player_name: RwSignal::new("Player".to_string()),
            difficulty: RwSignal::new(Difficulty::Medium),
            halfmove_clock: RwSignal::new(0),
            captured_white: RwSignal::new(Vec::new()),
            captured_black: RwSignal::new(Vec::new()),
            commentary: RwSignal::new(None),
            commentary_gen: RwSignal::new(0),
            persona: RwSignal::new(persona_for_difficulty(Difficulty::Medium)),
        }
    }

    pub fn reset(&self) {
        use crate::engine::persona::persona_for_difficulty;
        self.board.set(Board::starting_position());
        self.active_color.set(Color::White);
        self.phase.set(GamePhase::Playing);
        self.en_passant.set(None);
        self.castling.set(CastlingRights::all());
        self.move_history.set(Vec::new());
        self.selected_square.set(None);
        self.valid_moves_for_selected.set(Vec::new());
        self.last_move.set(None);
        self.engine_highlight.set(None);
        self.halfmove_clock.set(0);
        self.captured_white.set(Vec::new());
        self.captured_black.set(Vec::new());
        self.commentary.set(None);
        self.commentary_gen.set(0);
        // Re-initialize persona from current difficulty
        self.persona.set(persona_for_difficulty(self.difficulty.get_untracked()));
    }

    /// Try to select a square. Returns true if a piece was selected.
    pub fn select_square(&self, pos: Pos) -> bool {
        let board = self.board.get();
        let color = self.active_color.get();
        let ep = self.en_passant.get();
        let castling = self.castling.get();

        if let Some(piece) = board.get(pos) {
            if piece.color == color {
                let moves = legal_moves(&board, color, ep, castling)
                    .into_iter()
                    .filter(|m| m.from == pos)
                    .collect::<Vec<_>>();
                self.selected_square.set(Some(pos));
                self.valid_moves_for_selected.set(moves);
                return true;
            }
        }
        // Clicking empty or enemy square deselects
        self.selected_square.set(None);
        self.valid_moves_for_selected.set(Vec::new());
        false
    }

    /// Try to execute a move to `to` from the currently selected square.
    /// Returns true if the move was made.
    pub fn try_move_to(&self, to: Pos) -> bool {
        let valid = self.valid_moves_for_selected.get();

        // Find the move — prefer promotion to queen by default
        let mv = valid.iter().find(|m| {
            m.to == to && m.promotion.map_or(true, |p| {
                p == crate::state::piece::Promotion::Queen
            })
        }).copied();

        if let Some(mv) = mv {
            self.apply_move(mv);
            return true;
        }
        false
    }

    /// Apply a move and update all state.
    pub fn apply_move(&self, mv: Move) {
        let board = self.board.get();
        let color = self.active_color.get();
        let castling = self.castling.get();

        let captured = board.get(mv.to).or_else(|| {
            // En passant: captured pawn is on the mover's rank, target file
            if mv.is_en_passant {
                let cap_pos = crate::state::piece::Pos::new(mv.to.file, mv.from.rank);
                board.get(cap_pos)
            } else {
                None
            }
        });
        let piece = board.get(mv.from).expect("apply_move: no piece at from");

        let (new_board, new_ep) = apply_move_to_board(&board, &mv);
        let new_castling = update_castling_rights(castling, &mv);

        // Halfmove clock: reset on pawn move or capture
        let is_irreversible = piece.kind == crate::state::piece::PieceKind::Pawn || captured.is_some();
        self.halfmove_clock.update(|c| {
            if is_irreversible { *c = 0; } else { *c += 1; }
        });

        // Record captured piece
        if let Some(cap) = captured {
            match color {
                Color::White => self.captured_white.update(|v| v.push(cap)),
                Color::Black => self.captured_black.update(|v| v.push(cap)),
            }
        }

        // Move history
        let record = MoveRecord {
            mv,
            notation: mv.to_notation(),
            piece,
            captured,
        };
        self.move_history.update(|h| h.push(record));

        // Apply board
        self.board.set(new_board.clone());
        self.en_passant.set(new_ep);
        self.castling.set(new_castling);
        self.last_move.set(Some(mv));
        self.selected_square.set(None);
        self.valid_moves_for_selected.set(Vec::new());

        let next_color = color.opposite();

        // Determine new phase
        let new_phase = if self.halfmove_clock.get() >= 100 {
            GamePhase::DrawFiftyMove
        } else if is_checkmate(&new_board, next_color, new_ep, new_castling) {
            GamePhase::Checkmate
        } else if is_stalemate(&new_board, next_color, new_ep, new_castling) {
            GamePhase::Stalemate
        } else if is_in_check(&new_board, next_color) {
            GamePhase::Check
        } else {
            GamePhase::Playing
        };

        self.phase.set(new_phase);
        self.active_color.set(next_color);
    }

    /// Is the given square the source or destination of the last move?
    pub fn is_last_move_square(&self, pos: Pos) -> bool {
        self.last_move
            .get()
            .map_or(false, |m| m.from == pos || m.to == pos)
    }

    /// Is the given square a valid move destination for the selected piece?
    pub fn is_valid_target(&self, pos: Pos) -> bool {
        self.valid_moves_for_selected
            .get()
            .iter()
            .any(|m| m.to == pos)
    }

    /// Is the game over?
    pub fn is_game_over(&self) -> bool {
        matches!(
            self.phase.get(),
            GamePhase::Checkmate | GamePhase::Stalemate | GamePhase::DrawFiftyMove
        )
    }

    /// Set commentary and bump the generation counter (drives animation re-trigger in UI).
    pub fn set_commentary(&self, line: String) {
        self.commentary.set(Some(line));
        self.commentary_gen.update(|g| *g = g.wrapping_add(1));
    }

    /// Clear commentary (hides the bubble).
    pub fn clear_commentary(&self) {
        self.commentary.set(None);
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
