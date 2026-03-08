# Task M2-T1: Game State & Scoring Engine

**Milestone:** M2 — Game State & Scoring Engine
**Status:** ✅ Done

## Restatement

This task implements the complete pure-Rust game state model and scoring logic for rw_sixzee in `src/state/game.rs` and `src/state/scoring.rs`. It covers the `GameState` struct, all 13 scoring functions, column/grand total computations, the full turn lifecycle (`start_turn`, `roll`, `detect_bonus_sixzee`, `place_score`), and the `CompletedGame` struct. No UI or Leptos signals are involved — this is a pure data/logic layer fully testable via `cargo test` on the native target. The DP-based scoring advisor (M4) and localStorage persistence (M5) are explicitly out of scope.

## Design

### Data flow

- `GameState` is a plain struct: `cells: [[Option<u8>; 13]; 6]`, `dice: [Option<u8>; 5]`, `held: [bool; 5]`, `rolls_used: u8`, `turn: u32`, `bonus_turn: bool`, `bonus_pool: u32`, `bonus_forfeited: bool`, `id: String`, `started_at: String`.
- `scoring.rs` functions take `[u8; 5]` (all dice present; callers unwrap).
- `game.rs` lifecycle: `new_game()` → `start_turn()` → `roll()` → `detect_bonus_sixzee()` → `place_score()` → loop or game complete.
- `score_preview_all(state)` returns `[[u8; 13]; 6]` of what the current dice would score in each cell (0 if dice not all `Some`).

### Function / type signatures

```rust
// scoring.rs
pub fn score_ones(dice: [u8; 5]) -> u8
pub fn score_twos(dice: [u8; 5]) -> u8
pub fn score_threes(dice: [u8; 5]) -> u8
pub fn score_fours(dice: [u8; 5]) -> u8
pub fn score_fives(dice: [u8; 5]) -> u8
pub fn score_sixes(dice: [u8; 5]) -> u8
pub fn score_three_of_a_kind(dice: [u8; 5]) -> u8
pub fn score_four_of_a_kind(dice: [u8; 5]) -> u8
pub fn score_full_house(dice: [u8; 5]) -> u8
pub fn score_small_straight(dice: [u8; 5]) -> u8
pub fn score_large_straight(dice: [u8; 5]) -> u8
pub fn score_sixzee(dice: [u8; 5]) -> u8
pub fn score_chance(dice: [u8; 5]) -> u8
pub fn score_for_row(row: usize, dice: [u8; 5]) -> u8
pub fn upper_subtotal(col: &[Option<u8>; 13]) -> u16
pub fn upper_bonus(col: &[Option<u8>; 13]) -> u16
pub fn lower_subtotal(col: &[Option<u8>; 13]) -> u16
pub fn column_total(col: &[Option<u8>; 13]) -> u16
pub fn grand_total(cells: &[[Option<u8>; 13]; 6], bonus: u32) -> u32

// game.rs
pub struct GameState { ... }
pub struct CompletedGame { ... }
pub fn new_game() -> GameState
pub fn start_turn(state: &mut GameState)
pub fn roll(state: &mut GameState) -> AppResult<()>
pub fn detect_bonus_sixzee(state: &mut GameState)
pub fn place_score(state: &mut GameState, col: usize, row: usize) -> AppResult<()>
pub fn is_game_complete(state: &GameState) -> bool
pub fn score_preview_all(state: &GameState) -> [[u8; 13]; 6]
```

### Edge cases

- `detect_bonus_sixzee`: must require ALL 6 `cells[col][11]` are `Some(_)`, not just ≥1. Also checks all 5 dice are equal.
- `score_full_house`: 5-of-a-kind is NOT a full house (no 2+3 split). Must return 0.
- `score_small_straight`: sorted-unique set. `{1,2,3,4,5}` and `{2,3,4,5,6}` both qualify.
- `bonus_forfeited`: set on first placement of `Some(0)` in row 11. Never reset.
- `bonus_pool`: incremented by 100 per bonus Sixzee only if `!bonus_forfeited`.
- `score_preview_all`: returns all zeros when any die is `None`.
- `place_score`: calling with already-filled cell is a programming error → `AppError::Internal`.
- `roll` called with `rolls_used == 3` is a UI concern; no enforcement in this layer.

### Integration points

- `src/state/mod.rs` — add `pub mod game; pub mod scoring;`
- `src/state/game.rs` — new file
- `src/state/scoring.rs` — new file
- `src/error.rs` — already defines `AppError::Internal`

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `detect_bonus_sixzee` calls `start_turn` which resets dice; is the bonus pool increment lost? | No: pool is incremented before `start_turn` clears dice/held only, not pool. |
| Simplicity | `score_for_row` dispatcher duplicates the row-index mapping | Row constants with doc comments make it self-documenting. |
| Coupling | `roll` increments `rolls_used` and calls `detect_bonus_sixzee`; could be separate | The turn lifecycle requires this ordering; keeping in one fn is correct. |
| Performance | Scoring iterates dice up to 3x per preview call | O(5×13×6) = O(390) per preview — negligible for browser use. |
| Testability | `roll` uses `rand`; hard to test exact dice values | Test `rolls_used` increment and that held dice are unchanged; not dice values. |

## Implementation Notes

- Row index constants defined as `pub const ROW_*: usize` at top of `game.rs`.
- `new_game()` uses `uuid::Uuid::new_v4()` for `id` and `js-sys Date` on wasm / `std::time` on native for `started_at`. Since `started_at` is cosmetic (history display), a simple placeholder is acceptable for native tests.
- `rand::thread_rng().gen_range(1..=6u8)` for dice rolls.
- `score_small_straight`: collect unique values into a sorted Vec, then check for 4 consecutive.
- `score_full_house`: count value frequencies; require exactly one pair and one triple.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| score_ones / twos / threes / fours / fives / sixes | 1 | ✅ | multiple dice combos each |
| score_three_of_a_kind valid / zero | 1 | ✅ | |
| score_four_of_a_kind valid / zero | 1 | ✅ | |
| score_full_house valid / zero / 5-of-a-kind | 1 | ✅ | edge case: 5-of-a-kind → 0 |
| score_small_straight all 3 variants / zero | 1 | ✅ | |
| score_large_straight both variants / zero | 1 | ✅ | |
| score_sixzee all-same / not-same | 1 | ✅ | |
| score_chance | 1 | ✅ | |
| upper_bonus at 62 / 63 / 64 | 1 | ✅ | |
| grand_total with known multi-col + bonus pool | 1 | ✅ | |
| detect_bonus_sixzee fires when all 6 Sixzee cells filled | 1 | ✅ | |
| detect_bonus_sixzee does NOT fire when any Sixzee cell None | 1 | ✅ | |
| bonus_pool not incremented after bonus_forfeited | 1 | ✅ | |
| bonus_forfeited set on place_score(col, 11) with zero dice | 1 | ✅ | |
| start_turn clears dice/held/rolls_used | 1 | ✅ | |
| score_preview_all with None dice → all zeros | 1 | ✅ | |
| is_game_complete false/true | 1 | ✅ | |
| roll increments rolls_used, held dice unchanged | 1 | ✅ | |
| place_score on already-filled cell → AppError::Internal | 1 | ✅ | |

## Test Results

`cargo test`: 56 passed, 0 failed (52 new tests added by M2, 4 pre-existing router tests).
`cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings`: clean.
`trunk build`: success.

## Review Notes

Code-review agent found no bugs or logic errors. All 7 constraint checks passed.

The `detect_bonus_sixzee` bonus_turn flag is set and then immediately cleared by `start_turn`. The UI (M5) will read `bonus_pool` delta to surface the bonus message, which is consistent with the tech spec §5.3.

## Callouts / Gotchas

- `score_full_house` uses `value_counts` which returns per-value frequency counts [1..6]. The check `contains(&3) && contains(&2)` correctly returns 0 for 5-of-a-kind (counts = [5,0,...] — no 2 present).
- `score_small_straight` iterates `start in 1..=3` (starts 1, 2, 3) checking runs of 4. This correctly covers `{1,2,3,4}`, `{2,3,4,5}`, `{3,4,5,6}`.
- `current_iso8601()` on native target returns a fixed sentinel. This is intentional and documented — `started_at` is cosmetic and only displayed in history (M6).
- `detect_bonus_sixzee` resets dice via `start_turn` immediately after crediting the bonus pool. The bonus pool increment happens before the reset, so no value is lost.
