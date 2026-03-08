# M2 — Game State & Scoring Engine

<!-- MILESTONE: M2 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** [M1 — Project Bootstrap](m1-bootstrap.md)
**Required by:** M4, M5, M6, M7

---

## Overview

Implement the complete game state model and all scoring logic as pure Rust. This milestone contains no UI — it produces
a fully-tested `state/` module that encodes every rule from the PRD: all 13 scoring categories, upper-section bonus,
Sixzee bonus pool, turn lifecycle, and grand total calculation. Native `cargo test` provides verification without a
browser.

---

## Success Criteria

- [ ] `python make.py test` passes with zero test failures; all tests run natively (no WASM)
- [ ] All 13 scoring functions return correct values for representative dice combinations including
  edge cases (all-same, sequential, zero-score cases)
- [ ] `upper_bonus()` returns 35 at ≥63, 0 at 62 and below
- [ ] `grand_total()` correctly sums 6 column totals and the bonus pool
- [ ] `detect_bonus_sixzee()` fires only when all 5 dice match AND all 6 Sixzee cells (row 11) are `Some(_)`;
  adds 100 to bonus pool if not forfeited; calls `start_turn()` immediately
- [ ] Scratching a Sixzee cell (placing `Some(0)` in row 11) sets `bonus_forfeited = true` permanently
- [ ] `place_score()` correctly fills the cell, advances `turn`, and calls `start_turn()`
- [ ] `start_turn()` resets dice to `[None; 5]`, held to `[false; 5]`, `rolls_used` to 0, `bonus_turn` to false
- [ ] `roll()` only re-randomises un-held dice; held dice values are unchanged
- [ ] After `roll()` is called 3 times in a turn, `rolls_used == 3` (enforced in UI, not here)
- [ ] Score persistence round-trip: `GameState` serialises to JSON and deserialises back with identical field values

---

## Tasks

### Data Model

- [ ] Implement `GameState` struct in `src/state/game.rs` with fields:
  `id: String`, `cells: [[Option<u8>; 13]; 6]`, `dice: [Option<u8>; 5]`, `held: [bool; 5]`,
  `rolls_used: u8`, `turn: u32`, `bonus_turn: bool`, `bonus_pool: u32`,
  `bonus_forfeited: bool`, `started_at: String`
- [ ] Derive `Serialize`, `Deserialize`, `Clone` on `GameState`
- [ ] Implement `GameState::new()` — sets `id` (uuid v4), `started_at` (ISO 8601), all cells `None`,
  dice `[None; 5]`, held `[false; 5]`, rolls_used/turn/bonus_pool all zero, bonus_forfeited false
- [ ] Define row-index constants (or doc comments) for the 13 row indices 0–12

### Scoring Functions (`src/state/scoring.rs`)

- [ ] Implement `score_ones(dice: [u8; 5]) -> u8` — sum of dice showing 1
- [ ] Implement `score_twos` through `score_sixes` — sum of dice showing N (rows 1–5)
- [ ] Implement `score_three_of_a_kind(dice: [u8; 5]) -> u8` — sum all 5 dice if ≥3 same, else 0
- [ ] Implement `score_four_of_a_kind(dice: [u8; 5]) -> u8` — sum all 5 dice if ≥4 same, else 0
- [ ] Implement `score_full_house(dice: [u8; 5]) -> u8` — 25 if exactly 3+2 distribution, else 0
- [ ] Implement `score_small_straight(dice: [u8; 5]) -> u8` — 30 if contains any 4 sequential values, else 0
- [ ] Implement `score_large_straight(dice: [u8; 5]) -> u8` — 40 if all 5 sequential (1-5 or 2-6), else 0
- [ ] Implement `score_sixzee(dice: [u8; 5]) -> u8` — 50 if all 5 same, else 0
- [ ] Implement `score_chance(dice: [u8; 5]) -> u8` — sum of all 5 dice unconditionally
- [ ] Implement `score_for_row(row: usize, dice: [u8; 5]) -> u8` — dispatcher over rows 0–12
- [ ] Implement `upper_subtotal(col: &[Option<u8>; 13]) -> u16` — sum of rows 0–5 (skipping `None`)
- [ ] Implement `upper_bonus(col: &[Option<u8>; 13]) -> u16` — 35 if upper_subtotal ≥ 63, else 0
- [ ] Implement `lower_subtotal(col: &[Option<u8>; 13]) -> u16` — sum of rows 6–12 (skipping `None`)
- [ ] Implement `column_total(col: &[Option<u8>; 13]) -> u16`
  = `upper_subtotal + upper_bonus + lower_subtotal`
- [ ] Implement `grand_total(cells: &[[Option<u8>; 13]; 6], bonus: u32) -> u32`
  = sum of all 6 `column_total` values plus `bonus`

### Turn Lifecycle (`src/state/game.rs`)

- [ ] Implement `start_turn(state: &mut GameState)` — resets dice, held, rolls_used, bonus_turn
- [ ] Implement `roll(state: &mut GameState) -> AppResult<()>` — re-rolls un-held dice (rand 1..=6),
  increments rolls_used, calls `detect_bonus_sixzee()`; storage persist deferred to M5
- [ ] Implement `detect_bonus_sixzee(state: &mut GameState)` — checks all 5 dice same value AND
  all 6 cells[col][11] are `Some(_)`; if true, sets `bonus_turn = true`, conditionally adds 100 to
  `bonus_pool`, calls `start_turn()`
- [ ] Implement `place_score(state: &mut GameState, col: usize, row: usize) -> AppResult<()>` —
  writes `score_for_row(row, current_dice)` to `cells[col][row]`; if row == 11 and score == 0,
  sets `bonus_forfeited = true`; increments `turn`; calls `start_turn()`; storage persist deferred to M5
- [ ] Implement `is_game_complete(state: &GameState) -> bool` — all 78 cells are `Some(_)`
- [ ] Implement `score_preview_all(state: &GameState) -> [[u8; 13]; 6]` — for each cell, returns
  `score_for_row(row, dice)` if dice are all `Some`; 0 otherwise (used by UI Memo)

### `CompletedGame` Struct

- [ ] Define `CompletedGame` struct in `src/state/game.rs` (or `storage.rs`):
  `id`, `completed_at`, `final_score`, `bonus_pool`, `bonus_forfeited`, `cells`
- [ ] Derive `Serialize`, `Deserialize`, `Clone`

### Unit Tests

- [ ] Test each scoring function with at least 3 dice combinations: valid score, zero (requirement unmet),
  and boundary/edge case (e.g. full house with 5-of-a-kind should return 0)
- [ ] Test `upper_bonus` at exactly 62, 63, and 64
- [ ] Test `grand_total` with known values across all 6 columns and a non-zero bonus pool
- [ ] Test `detect_bonus_sixzee` — fires when all 6 Sixzee cells filled; does NOT fire when any is `None`
- [ ] Test `bonus_forfeited` is set true when `place_score(col, 11)` with a zero-scoring dice set
- [ ] Test `bonus_pool` does NOT increment after `bonus_forfeited = true`
- [ ] Test `start_turn` clears dice/held/rolls_used correctly
- [ ] Test `score_preview_all` with unrolled state (all `None` dice) returns all-zero array

---

## Notes & Risks

- **`u8` overflow:** Upper subtotal can reach 6 × 30 = 180 (sixes × 5 dice × 6 rows) per column, fitting `u16`.
  Individual cell scores max at 50 (Sixzee), fitting `u8`. `grand_total` uses `u32` — sufficient.
- **Small straight detection:** A sorted-unique approach avoids off-by-one errors. Sets `{1,2,3,4}`,
  `{2,3,4,5}`, `{3,4,5,6}` should all return 30.
- **Dice representation:** Inside scoring functions, dice are always `[u8; 5]` with values 1–6.
  `GameState.dice` uses `Option<u8>` where `None` = unrolled. Callers must unwrap before scoring
  (permitted `expect()` site per §15.4 of tech spec).
