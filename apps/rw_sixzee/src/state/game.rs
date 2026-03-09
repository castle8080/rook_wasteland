//! Game state model and turn lifecycle for rw_sixzee.
//!
//! `GameState` is the single serialisable blob stored in localStorage (M5).
//! All mutation is done through the free functions in this module, which
//! keep the invariants described in the tech spec §5.3.

use crate::error::{AppError, AppResult};
use crate::state::scoring::{grand_total, score_for_row, ROW_COUNT, ROW_SIXZEE};
use rand::Rng;
use serde::{Deserialize, Serialize};

// ─── GameState ───────────────────────────────────────────────────────────────

/// Full serialisable game state. One blob per in-progress game in localStorage.
///
/// `cells[col][row]`: `None` = empty (not yet scored), `Some(v)` = filled.
/// Row indices are defined as `ROW_*` constants in `state::scoring`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
    /// UUID v4 generated at game start; also used as the history entry id.
    pub id: String,
    /// Scored cells: `[col 0..5][row 0..12]`.
    pub cells: [[Option<u8>; ROW_COUNT]; 6],
    /// Current dice values. `None` = not yet rolled this turn.
    pub dice: [Option<u8>; 5],
    /// Whether each die is held (preserved across rolls).
    pub held: [bool; 5],
    /// Number of rolls used in the current turn (0–3).
    pub rolls_used: u8,
    /// Turn counter (0-indexed, increments after each `place_score`).
    pub turn: u32,
    /// `true` during a bonus Sixzee turn; cleared by `start_turn`.
    pub bonus_turn: bool,
    /// Accumulated bonus pool (multiples of 100).
    pub bonus_pool: u32,
    /// Once `true`, no further bonus Sixzees are credited.
    pub bonus_forfeited: bool,
    /// ISO 8601 timestamp of game creation.
    pub started_at: String,
}

// ─── CompletedGame ───────────────────────────────────────────────────────────

/// Snapshot of a finished game stored in the history list in localStorage.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CompletedGame {
    /// Same UUID as the originating `GameState`.
    pub id: String,
    /// ISO 8601 timestamp of game completion.
    pub completed_at: String,
    /// Grand total score at game end.
    pub final_score: u32,
    /// Bonus pool at game end.
    pub bonus_pool: u32,
    /// Whether the Sixzee bonus was forfeited during this game.
    pub bonus_forfeited: bool,
    /// Final cell values.
    pub cells: [[Option<u8>; ROW_COUNT]; 6],
}

// ─── Constructor ─────────────────────────────────────────────────────────────

/// Create a fresh game with a new UUID and current timestamp.
pub fn new_game() -> GameState {
    GameState {
        id: new_uuid(),
        cells: [[None; ROW_COUNT]; 6],
        dice: [None; 5],
        held: [false; 5],
        rolls_used: 0,
        turn: 0,
        bonus_turn: false,
        bonus_pool: 0,
        bonus_forfeited: false,
        started_at: current_iso8601(),
    }
}

// ─── Turn lifecycle ───────────────────────────────────────────────────────────

/// Reset per-turn fields. Called at the start of every turn, including bonus turns.
pub fn start_turn(state: &mut GameState) {
    state.dice = [None; 5];
    state.held = [false; 5];
    state.rolls_used = 0;
    state.bonus_turn = false;
}

/// Roll all un-held dice, increment `rolls_used`, then detect bonus Sixzee.
///
/// Storage persistence is deferred to M5. The caller is responsible for
/// enforcing the 3-roll limit before calling this function.
pub fn roll(state: &mut GameState) -> AppResult<()> {
    let mut rng = rand::thread_rng();
    for i in 0..5 {
        if !state.held[i] {
            state.dice[i] = Some(rng.gen_range(1..=6u8));
        }
    }
    state.rolls_used += 1;
    detect_bonus_sixzee(state);
    Ok(())
}

/// Place a score in cell `(col, row)` using the current dice.
///
/// - Writes `score_for_row(row, current_dice)` into `cells[col][row]`.
/// - Sets `bonus_forfeited = true` if row 11 (Sixzee) is scored with 0.
/// - Increments `turn` and calls `start_turn()`.
/// - Storage persistence and end-game trigger are deferred to M5.
///
/// # Errors
///
/// Returns `AppError::Internal` if the cell is already filled or if the
/// current dice are not all rolled (any `None`).
pub fn place_score(state: &mut GameState, col: usize, row: usize) -> AppResult<()> {
    if col >= 6 || row >= ROW_COUNT {
        return Err(AppError::Internal("place_score: col or row out of range"));
    }
    if state.cells[col][row].is_some() {
        return Err(AppError::Internal("place_score: cell is already filled"));
    }
    let dice = current_dice(state)
        .ok_or(AppError::Internal("place_score: dice not fully rolled"))?;

    let score = score_for_row(row, dice);
    state.cells[col][row] = Some(score);

    if row == ROW_SIXZEE && score == 0 {
        state.bonus_forfeited = true;
    }

    state.turn += 1;
    start_turn(state);
    Ok(())
}

/// Check whether a bonus Sixzee has occurred and, if so, credit the pool and
/// start a new turn immediately.
///
/// Conditions:
/// 1. All 5 dice show the same value.
/// 2. All 6 `cells[col][ROW_SIXZEE]` are `Some(_)` (fully scored, any value).
///
/// When both conditions hold: sets `bonus_turn = true`, increments `bonus_pool`
/// by 100 (unless forfeited), then calls `start_turn`.
pub fn detect_bonus_sixzee(state: &mut GameState) {
    let Some(dice) = current_dice(state) else {
        return;
    };
    if !dice.iter().all(|&d| d == dice[0]) {
        return;
    }
    let all_sixzee_cells_filled = state
        .cells
        .iter()
        .all(|col| col[ROW_SIXZEE].is_some());
    if !all_sixzee_cells_filled {
        return;
    }
    state.bonus_turn = true;
    if !state.bonus_forfeited {
        state.bonus_pool += 100;
    }
    start_turn(state);
}

/// Returns `true` when all 78 cells (`6 × 13`) are filled.
pub fn is_game_complete(state: &GameState) -> bool {
    state.cells.iter().all(|col| col.iter().all(|c| c.is_some()))
}

// ─── History helpers ─────────────────────────────────────────────────────────

/// Sort a history list in-place by `final_score` descending.
pub fn sort_history_by_score(history: &mut [CompletedGame]) {
    history.sort_by(|a, b| b.final_score.cmp(&a.final_score));
}

/// Remove entries whose `completed_at` timestamp is more than 365 days before
/// `now_ms` (milliseconds since Unix epoch).
///
/// Entries whose timestamp cannot be parsed are retained (fail-safe).
pub fn prune_old_entries(history: Vec<CompletedGame>, now_ms: f64) -> Vec<CompletedGame> {
    const DAYS_365_MS: f64 = 365.0 * 24.0 * 3600.0 * 1000.0;
    let cutoff = now_ms - DAYS_365_MS;
    history
        .into_iter()
        .filter(|entry| {
            parse_timestamp_ms(&entry.completed_at)
                .map(|ts| ts > cutoff)
                .unwrap_or(true) // retain if unparseable
        })
        .collect()
}

/// Build a `CompletedGame` snapshot from a finished `GameState`.
///
/// Call only when `is_game_complete(state)` is `true`.
pub fn completed_game_from_state(state: &GameState) -> CompletedGame {
    CompletedGame {
        id: state.id.clone(),
        completed_at: current_iso8601(),
        final_score: grand_total(&state.cells, state.bonus_pool),
        bonus_pool: state.bonus_pool,
        bonus_forfeited: state.bonus_forfeited,
        cells: state.cells,
    }
}

/// For each cell `(col, row)`, returns the score the current dice would yield.
///
/// Returns all zeros if any die is `None` (unrolled state).
pub fn score_preview_all(state: &GameState) -> [[u8; ROW_COUNT]; 6] {
    let Some(dice) = current_dice(state) else {
        return [[0; ROW_COUNT]; 6];
    };
    let mut out = [[0u8; ROW_COUNT]; 6];
    for (col_idx, col) in state.cells.iter().enumerate() {
        for row in 0..ROW_COUNT {
            if col[row].is_none() {
                out[col_idx][row] = score_for_row(row, dice);
            }
        }
    }
    out
}

// ─── Private helpers ──────────────────────────────────────────────────────────

/// Returns `Some([u8; 5])` if all dice are rolled, else `None`.
pub fn current_dice(state: &GameState) -> Option<[u8; 5]> {
    let d0 = state.dice[0]?;
    let d1 = state.dice[1]?;
    let d2 = state.dice[2]?;
    let d3 = state.dice[3]?;
    let d4 = state.dice[4]?;
    Some([d0, d1, d2, d3, d4])
}

/// Parse an ISO 8601 timestamp string to milliseconds since Unix epoch.
///
/// On WASM, delegates to `js_sys::Date` for exact results.
/// On native (test) targets, uses an approximate formula good enough for
/// pruning comparisons (month lengths approximated as 30 days; accurate to
/// within ~5 days for any date in the current century).
fn parse_timestamp_ms(s: &str) -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        let d = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(s));
        let ms = d.get_time();
        if ms.is_nan() {
            None
        } else {
            Some(ms)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if s.len() < 10 {
            return None;
        }
        let year: i64 = s[0..4].parse().ok()?;
        let month: i64 = s[5..7].parse().ok()?;
        let day: i64 = s[8..10].parse().ok()?;
        // Days since Unix epoch (approximate: 30 days/month, 365+leap days/year)
        let days = (year - 1970) * 365 + (year - 1969) / 4 + (month - 1) * 30 + day - 1;
        Some((days * 86_400_000) as f64)
    }
}

/// Generate a new UUID v4 string.
fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Return an ISO 8601 timestamp string. On WASM, uses `js_sys::Date`; on
/// native, falls back to a fixed sentinel string that satisfies the type
/// contract without importing `std::time` into the wasm32 build.
fn current_iso8601() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::new_0().to_iso_string().into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Native target (cargo test). An accurate timestamp is not required
        // for test correctness; a fixed sentinel is acceptable here.
        "1970-01-01T00:00:00.000Z".to_string()
    }
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::scoring::{ROW_CHANCE, ROW_COUNT, ROW_ONES, ROW_SIXES, ROW_SIXZEE};

    fn filled_game() -> GameState {
        let mut g = new_game();
        for col in &mut g.cells {
            for cell in col.iter_mut() {
                *cell = Some(10);
            }
        }
        g
    }

    fn game_with_all_sixzee_filled(score: u8) -> GameState {
        let mut g = new_game();
        for col in &mut g.cells {
            col[ROW_SIXZEE] = Some(score);
        }
        g
    }

    fn make_completed(final_score: u32, id: &str) -> CompletedGame {
        CompletedGame {
            id: id.to_string(),
            completed_at: "2025-01-01T00:00:00.000Z".to_string(),
            final_score,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        }
    }

    // ── new_game / start_turn ──

    #[test]
    fn new_game_is_empty() {
        let g = new_game();
        assert_eq!(g.cells, [[None; ROW_COUNT]; 6]);
        assert_eq!(g.dice, [None; 5]);
        assert_eq!(g.held, [false; 5]);
        assert_eq!(g.rolls_used, 0);
        assert_eq!(g.turn, 0);
        assert!(!g.bonus_forfeited);
        assert_eq!(g.bonus_pool, 0);
    }

    #[test]
    fn start_turn_clears_per_turn_fields() {
        let mut g = new_game();
        g.dice = [Some(3); 5];
        g.held = [true; 5];
        g.rolls_used = 3;
        g.bonus_turn = true;
        start_turn(&mut g);
        assert_eq!(g.dice, [None; 5]);
        assert_eq!(g.held, [false; 5]);
        assert_eq!(g.rolls_used, 0);
        assert!(!g.bonus_turn);
        // Pool and forfeited are NOT cleared by start_turn
        g.bonus_pool = 100;
        g.bonus_forfeited = true;
        start_turn(&mut g);
        assert_eq!(g.bonus_pool, 100);
        assert!(g.bonus_forfeited);
    }

    // ── roll ──

    #[test]
    fn roll_increments_rolls_used() {
        let mut g = new_game();
        roll(&mut g).expect("roll should succeed");
        assert_eq!(g.rolls_used, 1);
        roll(&mut g).expect("roll should succeed");
        assert_eq!(g.rolls_used, 2);
    }

    #[test]
    fn roll_does_not_change_held_dice() {
        let mut g = new_game();
        // Set dice manually and hold die 2.
        g.dice = [Some(4), Some(4), Some(4), Some(4), Some(4)];
        g.held[2] = true;
        g.rolls_used = 1;
        roll(&mut g).expect("roll should succeed");
        assert_eq!(g.dice[2], Some(4));
    }

    #[test]
    fn roll_gives_all_dice_valid_values() {
        let mut g = new_game();
        roll(&mut g).expect("roll should succeed");
        for d in g.dice {
            let v = d.expect("die should have a value after roll");
            assert!((1..=6).contains(&v));
        }
    }

    // ── place_score ──

    #[test]
    fn place_score_fills_cell_and_advances_turn() {
        let mut g = new_game();
        g.dice = [Some(1), Some(1), Some(2), Some(3), Some(4)];
        g.rolls_used = 1;
        let before_turn = g.turn;
        place_score(&mut g, 0, ROW_ONES).expect("place_score should succeed");
        assert_eq!(g.cells[0][ROW_ONES], Some(2)); // 1+1
        assert_eq!(g.turn, before_turn + 1);
        // start_turn was called
        assert_eq!(g.dice, [None; 5]);
    }

    #[test]
    fn place_score_returns_error_on_filled_cell() {
        let mut g = new_game();
        g.cells[0][ROW_ONES] = Some(3);
        g.dice = [Some(1), Some(1), Some(1), Some(2), Some(3)];
        g.rolls_used = 1;
        let result = place_score(&mut g, 0, ROW_ONES);
        assert!(matches!(result, Err(AppError::Internal(_))));
    }

    #[test]
    fn place_score_returns_error_when_dice_unrolled() {
        let mut g = new_game();
        // dice still None
        let result = place_score(&mut g, 0, ROW_ONES);
        assert!(matches!(result, Err(AppError::Internal(_))));
    }

    #[test]
    fn place_score_sets_bonus_forfeited_on_zero_sixzee() {
        let mut g = new_game();
        // Non-Sixzee dice → score_sixzee returns 0
        g.dice = [Some(1), Some(2), Some(3), Some(4), Some(5)];
        g.rolls_used = 1;
        assert!(!g.bonus_forfeited);
        place_score(&mut g, 0, ROW_SIXZEE).expect("place_score should succeed");
        assert!(g.bonus_forfeited);
    }

    #[test]
    fn place_score_does_not_forfeit_when_sixzee_scores() {
        let mut g = new_game();
        g.dice = [Some(3), Some(3), Some(3), Some(3), Some(3)];
        g.rolls_used = 1;
        place_score(&mut g, 0, ROW_SIXZEE).expect("place_score should succeed");
        assert!(!g.bonus_forfeited);
    }

    // ── detect_bonus_sixzee ──

    #[test]
    fn detect_bonus_sixzee_fires_when_all_sixzee_cells_filled() {
        let mut g = game_with_all_sixzee_filled(50);
        g.dice = [Some(5), Some(5), Some(5), Some(5), Some(5)];
        g.rolls_used = 1;
        detect_bonus_sixzee(&mut g);
        assert_eq!(g.bonus_pool, 100);
        // start_turn was called
        assert_eq!(g.dice, [None; 5]);
    }

    #[test]
    fn detect_bonus_sixzee_does_not_fire_when_any_sixzee_cell_none() {
        let mut g = game_with_all_sixzee_filled(50);
        // Leave one column's Sixzee cell empty
        g.cells[3][ROW_SIXZEE] = None;
        g.dice = [Some(2), Some(2), Some(2), Some(2), Some(2)];
        g.rolls_used = 1;
        detect_bonus_sixzee(&mut g);
        assert_eq!(g.bonus_pool, 0);
        assert_eq!(g.dice, [Some(2); 5]); // unchanged
    }

    #[test]
    fn detect_bonus_sixzee_does_not_fire_when_dice_not_all_same() {
        let mut g = game_with_all_sixzee_filled(50);
        g.dice = [Some(1), Some(2), Some(3), Some(4), Some(5)];
        g.rolls_used = 1;
        detect_bonus_sixzee(&mut g);
        assert_eq!(g.bonus_pool, 0);
    }

    #[test]
    fn detect_bonus_sixzee_does_not_credit_when_forfeited() {
        let mut g = game_with_all_sixzee_filled(0); // all zero Sixzee cells
        g.bonus_forfeited = true;
        g.dice = [Some(4), Some(4), Some(4), Some(4), Some(4)];
        g.rolls_used = 1;
        detect_bonus_sixzee(&mut g);
        // Pool should NOT have increased
        assert_eq!(g.bonus_pool, 0);
        // But bonus_turn should still have been set (before start_turn clears it)
        // start_turn was called so dice reset
        assert_eq!(g.dice, [None; 5]);
    }

    #[test]
    fn bonus_pool_accumulates_on_repeated_bonus_sixzees() {
        let mut g = game_with_all_sixzee_filled(50);
        for _ in 0..3 {
            g.dice = [Some(6), Some(6), Some(6), Some(6), Some(6)];
            g.rolls_used = 1;
            detect_bonus_sixzee(&mut g);
        }
        assert_eq!(g.bonus_pool, 300);
    }

    // ── is_game_complete ──

    #[test]
    fn is_game_complete_false_on_empty_board() {
        let g = new_game();
        assert!(!is_game_complete(&g));
    }

    #[test]
    fn is_game_complete_true_when_all_filled() {
        let g = filled_game();
        assert!(is_game_complete(&g));
    }

    #[test]
    fn is_game_complete_false_when_one_cell_missing() {
        let mut g = filled_game();
        g.cells[2][ROW_CHANCE] = None;
        assert!(!is_game_complete(&g));
    }

    // ── score_preview_all ──

    #[test]
    fn score_preview_all_zeros_when_dice_unrolled() {
        let g = new_game();
        let preview = score_preview_all(&g);
        assert_eq!(preview, [[0; ROW_COUNT]; 6]);
    }

    #[test]
    fn score_preview_all_returns_expected_scores() {
        let mut g = new_game();
        g.dice = [Some(1), Some(1), Some(1), Some(2), Some(3)];
        g.rolls_used = 1;
        let preview = score_preview_all(&g);
        // Row 0 (Ones) = 3 across all columns
        for col_scores in &preview {
            assert_eq!(col_scores[ROW_ONES], 3);
        }
    }

    #[test]
    fn score_preview_all_shows_zero_for_filled_cells() {
        let mut g = new_game();
        g.dice = [Some(1), Some(1), Some(1), Some(2), Some(3)];
        g.rolls_used = 1;
        // Fill col 0 row 0
        g.cells[0][ROW_ONES] = Some(3);
        let preview = score_preview_all(&g);
        assert_eq!(preview[0][ROW_ONES], 0); // already filled → 0
        assert_eq!(preview[1][ROW_ONES], 3); // unfilled → score
    }

    // ── roll → detect_bonus_sixzee integration ──

    #[test]
    fn roll_triggers_bonus_sixzee_when_held_dice_form_sixzee() {
        // Hold all five dice to the same value so roll() leaves them unchanged,
        // then confirm detect_bonus_sixzee fires via the roll() code path.
        let mut g = game_with_all_sixzee_filled(50);
        g.dice = [Some(4), Some(4), Some(4), Some(4), Some(4)];
        g.held = [true; 5];
        g.rolls_used = 1;
        roll(&mut g).expect("roll should succeed");
        // detect_bonus_sixzee was invoked inside roll: pool += 100, start_turn called
        assert_eq!(g.bonus_pool, 100);
        assert_eq!(g.dice, [None; 5]);
    }

    // ── place_score bounds ──

    #[test]
    fn place_score_returns_error_on_out_of_bounds_col() {
        let mut g = new_game();
        g.dice = [Some(1), Some(1), Some(1), Some(1), Some(1)];
        g.rolls_used = 1;
        assert!(matches!(
            place_score(&mut g, 6, ROW_ONES),
            Err(AppError::Internal(_))
        ));
    }

    #[test]
    fn place_score_returns_error_on_out_of_bounds_row() {
        let mut g = new_game();
        g.dice = [Some(1), Some(1), Some(1), Some(1), Some(1)];
        g.rolls_used = 1;
        assert!(matches!(
            place_score(&mut g, 0, ROW_COUNT),
            Err(AppError::Internal(_))
        ));
    }

    // ── JSON round-trip ──

    #[test]
    fn game_state_json_round_trip() {
        let mut g = new_game();
        g.cells[0][ROW_SIXES] = Some(24);
        g.bonus_pool = 200;
        g.bonus_forfeited = true;
        let json = serde_json::to_string(&g).expect("serialize GameState");
        let g2: GameState = serde_json::from_str(&json).expect("deserialize GameState");
        assert_eq!(g, g2);
    }

    #[test]
    fn completed_game_json_round_trip() {
        let cg = CompletedGame {
            id: "test-id".to_string(),
            completed_at: "2025-01-01T00:00:00Z".to_string(),
            final_score: 350,
            bonus_pool: 100,
            bonus_forfeited: false,
            cells: [[Some(5); ROW_COUNT]; 6],
        };
        let json = serde_json::to_string(&cg).expect("serialize CompletedGame");
        let cg2: CompletedGame = serde_json::from_str(&json).expect("deserialize CompletedGame");
        assert_eq!(cg, cg2);
    }

    // ── sort_history_by_score ──

    #[test]
    fn sort_history_by_score_orders_descending() {
        let mut h = vec![
            make_completed(100, "a"),
            make_completed(300, "b"),
            make_completed(200, "c"),
        ];
        sort_history_by_score(&mut h);
        assert_eq!(h[0].final_score, 300);
        assert_eq!(h[1].final_score, 200);
        assert_eq!(h[2].final_score, 100);
    }

    #[test]
    fn sort_history_empty_vec_does_not_panic() {
        let mut h: Vec<CompletedGame> = vec![];
        sort_history_by_score(&mut h);
        assert!(h.is_empty());
    }

    #[test]
    fn sort_history_stable_for_equal_scores() {
        // Two entries with the same score — should not panic; order is unspecified
        // but both must still be present.
        let mut h = vec![make_completed(200, "x"), make_completed(200, "y")];
        sort_history_by_score(&mut h);
        assert_eq!(h.len(), 2);
        assert!(h.iter().all(|e| e.final_score == 200));
    }

    // ── prune_old_entries ──
    // Uses now_ms = 1_735_689_600_000.0 ≈ 2025-01-01T00:00:00Z

    #[test]
    fn prune_old_entries_removes_entries_older_than_365_days() {
        // 2022-01-01 is ~3 years before 2025-01-01 → pruned
        let old = CompletedGame {
            id: "old".to_string(),
            completed_at: "2022-01-01T00:00:00.000Z".to_string(),
            final_score: 100,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        };
        let pruned = prune_old_entries(vec![old], 1_735_689_600_000.0);
        assert!(pruned.is_empty(), "entry >365 days old must be pruned");
    }

    #[test]
    fn prune_old_entries_retains_recent_entries() {
        // 2024-12-15 is ~16 days before 2025-01-01 → retained
        let recent = CompletedGame {
            id: "recent".to_string(),
            completed_at: "2024-12-15T00:00:00.000Z".to_string(),
            final_score: 200,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        };
        let pruned = prune_old_entries(vec![recent], 1_735_689_600_000.0);
        assert_eq!(pruned.len(), 1, "recent entry must be retained");
        assert_eq!(pruned[0].id, "recent");
    }

    #[test]
    fn prune_old_entries_removes_old_and_retains_recent() {
        let old = CompletedGame {
            id: "old".to_string(),
            completed_at: "2022-01-01T00:00:00.000Z".to_string(),
            final_score: 100,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        };
        let recent = CompletedGame {
            id: "recent".to_string(),
            completed_at: "2024-12-15T00:00:00.000Z".to_string(),
            final_score: 200,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        };
        let pruned = prune_old_entries(vec![old, recent], 1_735_689_600_000.0);
        assert_eq!(pruned.len(), 1);
        assert_eq!(pruned[0].id, "recent");
    }

    #[test]
    fn prune_old_entries_empty_history_returns_empty() {
        let pruned = prune_old_entries(vec![], 1_735_689_600_000.0);
        assert!(pruned.is_empty());
    }

    #[test]
    fn prune_old_entries_retains_unparseable_timestamps() {
        // Garbled timestamps should be retained (fail-safe)
        let garbled = CompletedGame {
            id: "garbled".to_string(),
            completed_at: "not-a-date".to_string(),
            final_score: 150,
            bonus_pool: 0,
            bonus_forfeited: false,
            cells: [[None; ROW_COUNT]; 6],
        };
        let pruned = prune_old_entries(vec![garbled], 1_735_689_600_000.0);
        assert_eq!(pruned.len(), 1, "entry with unparseable timestamp must be retained");
    }

    // ── mini-game: fill all 78 cells programmatically ──

    /// Simulate a complete Sixzee game by setting dice directly and placing a
    /// score in every cell.  After 78 placements (6 cols × 13 rows):
    /// - `is_game_complete()` must return `true`
    /// - `grand_total()` must be greater than 0
    ///
    /// This is the Tier 1 "mini-game" test required by the M10 acceptance criteria.
    #[test]
    fn mini_game_fill_all_cells_is_complete_with_nonzero_total() {
        let mut g = new_game();
        // Use dice [1,1,1,1,1] for every turn so score_for_row gives deterministic
        // non-zero values for upper rows (Ones = 5) and a non-zero Chance (5).
        // Lower-section rows that need specific patterns are scored with zeros —
        // that is fine; grand_total still ends up > 0 from the upper rows.
        for col in 0..6_usize {
            for row in 0..ROW_COUNT {
                // Set fresh dice before each placement (place_score requires them).
                g.dice = [Some(1); 5];
                g.rolls_used = 1;
                place_score(&mut g, col, row).expect("place_score must succeed for empty cell");
                // Reset turn state to allow the next placement (start_turn is
                // called inside place_score, but bonus_sixzee detection clears
                // dice; set dice again on next iteration).
            }
        }
        assert!(
            is_game_complete(&g),
            "all 78 cells filled → game must be complete"
        );
        let total = grand_total(&g.cells, g.bonus_pool);
        assert!(
            total > 0,
            "grand total must be > 0 after filling with [1,1,1,1,1] dice; got {total}"
        );
    }

    // ── completed_game_from_state ──

    #[test]
    fn completed_game_from_state_captures_fields() {
        let mut g = new_game();
        g.bonus_pool = 200;
        g.bonus_forfeited = false;
        g.cells[0][ROW_ONES] = Some(5);
        let cg = completed_game_from_state(&g);
        assert_eq!(cg.id, g.id);
        assert_eq!(cg.bonus_pool, 200);
        assert_eq!(cg.cells[0][ROW_ONES], Some(5));
        assert!(!cg.completed_at.is_empty());
    }

    #[test]
    fn completed_game_from_state_bonus_forfeited_preserved() {
        let mut g = new_game();
        g.bonus_forfeited = true;
        let cg = completed_game_from_state(&g);
        assert!(cg.bonus_forfeited);
    }
}

