# M4 — DP Precomputation

<!-- MILESTONE: M4 -->
<!-- STATUS: DONE -->

**Status:** ✅ DONE
**Depends on:** [M2 — Game State & Scoring Engine](m2-scoring-engine.md)
**Required by:** [M7 — Ask Grandma](m7-ask-grandma.md)

---

## Overview

Build the offline DP solver and produce the precomputed `V_COL` value table that powers Ask Grandma's
score-estimation engine. This milestone is placed early to validate the mathematical approach — specifically, that the
single-column DP recurrence produces values in the expected theoretical range — before any advisor UI is built.

The offline crate is a standalone pure-Rust binary (`offline/`). Its output (`generated/v_col.rs`) is committed to the
repo and included at compile time in the Worker binary. Running the solver is only needed when scoring rules change.

---

## Success Criteria

- [ ] `cd offline && cargo run --release` completes in under 5 seconds and produces `generated/v_col.rs`
- [ ] `V_COL[0b1_1111_1111_1111]` (all 13 bits set — all cells filled) equals `0.0` exactly
- [ ] `V_COL[0]` (no cells filled, empty column) is in the range **230.0 – 280.0** (theoretical single-column
  optimum for Sixzee is approximately 254; value should be in this neighborhood)
- [ ] `YZ_BONUS_CORRECTION[0]` (6 Sixzee cells open, bonus not forfeited) is positive and reasonable
  (expected bonus pool contribution when all cells available)
- [ ] `YZ_BONUS_CORRECTION[7]` (0 Sixzee cells open, not forfeited) equals `0.0`
  (no open cells, no future bonus possible)
- [ ] The generated file compiles into the Worker crate without error (validated in M7, but
  format/syntax must be correct)
- [ ] All 8,192 entries are finite (no NaN, no Inf) — enforced by an assertion in the solver

---

## Tasks

### Offline Crate Setup

- [x] Create `offline/Cargo.toml` as a standalone binary crate (no shared dependencies with main app;
  pure Rust std only)
- [x] Create `offline/src/main.rs` entry point

### Dice Multiset Enumeration

- [x] Enumerate all 252 distinct dice multisets for 5d6 (combinations with replacement)
- [x] For each multiset, compute its exact multinomial probability weight: `5! / (n1! * n2! * ... * n6!) / 6^5`
- [x] Verify: sum of all weights equals 1.0 (within floating-point tolerance)
- [x] Cache this set — it is reused in every DP computation step

### Hold-Mask Optimizer

- [x] For a given dice roll and hold mask, compute the distribution of re-rolled outcomes:
  enumerate all `6^k` outcomes for `k` unheld dice; weight by uniform probability
- [x] Implement deduplication: for re-roll evaluation, collapse hold masks that result in the same
  sorted tuple of held values (e.g. holding dice indices 0,2 showing [5,5] is identical to holding
  indices 1,3 showing [5,5])

### DP Backward Induction

- [x] Allocate `v_col: [f64; 8192]` (use f64 for intermediate precision; output as f32)
- [x] Iterate fill patterns from `0b1_1111_1111_1111` (13 bits, value 8191) down to 0
- [x] For each fill pattern, compute `V_col(fill)` via the recurrence:
  ```
  V_col(fill) = E_{dice} [ best_turn(fill, dice, rolls_remaining=3) ]
  ```
  where the expectation is over all 252 multisets weighted by their probabilities
- [x] `best_turn(fill, dice, r=0)` = max over open rows of `score(row, dice) + v_col[fill | 1 << row]`
- [x] `best_turn(fill, dice, r>0)` = max of:
  - scoring now: `best_turn(fill, dice, 0)`
  - for each unique hold strategy: `E_{reroll}[best_turn(fill, new_dice, r-1)]`
- [x] Assert all 8,192 values are finite after computation
- [x] Cast f64 values to f32 for output

### Sixzee Bonus Correction Table

- [x] Compute `YZ_BONUS_CORRECTION: [f32; 14]` — indexed by `(n_sixzee_open: 0–6, forfeited: bool)`
  (7 values × 2 states = 14 entries; forfeited=false in indices 0–6, forfeited=true in indices 7–13)
- [x] For each `n_sixzee_open`, estimate the expected bonus pool contribution assuming optimal play:
  approximate via the probability of rolling Sixzee at least once per turn × 100, summed over
  remaining Sixzee-eligible turns (simplified closed-form is acceptable; exact DP over Sixzee
  sub-game is preferred but not required)
- [x] `YZ_BONUS_CORRECTION[7..=13]` (forfeited=true) all equal `0.0`

### Code Generation

- [x] Write `generated/v_col.rs` containing:
  ```rust
  pub const V_COL: [f32; 8192] = [ /* 8192 comma-separated values */ ];
  pub const YZ_BONUS_CORRECTION: [f32; 14] = [ /* 14 values */ ];
  ```
- [x] Ensure the file is valid Rust source (round-trip: `rustfmt` the output if needed, or verify
  it compiles with `rustc --edition 2021 --crate-type lib generated/v_col.rs`)
- [x] Commit `generated/v_col.rs` to the repository

### Validation Assertions (in solver)

- [x] Assert `v_col[8191] == 0.0`
- [x] Assert `v_col[0]` is in range [200.0, 300.0] and print the value
- [x] Assert all values are finite
- [x] Print a summary: min value, max value, and `v_col[0]` for manual inspection

---

## Notes & Risks

- **Runtime:** Pure backward induction with 8,192 patterns × 252 dice multisets × ~32 hold masks per die set
  should complete in well under 1 second on a modern CPU. If unexpectedly slow, profile the hold-mask
  enumeration inner loop.
- **f64 vs f32:** Use f64 throughout internal computation to avoid accumulated rounding error;
  cast to f32 only when writing the output array. The 32 KB embedded size target requires f32.
- **Scoring functions reuse:** The offline solver must replicate the same scoring logic as M2's
  `score_for_row()`. Consider sharing the logic via a common crate feature or by copying the
  scoring module. Divergence here will cause the advisor to recommend suboptimal moves.
- **Sixzee bonus correction:** The main app uses this table additively during advisor ranking.
  If the correction is materially wrong, advisor quality degrades but the game remains playable.
  A reasonable approximation is sufficient at this stage; it can be refined later.
- **Committed artifact:** `generated/v_col.rs` is committed so the main build has no dependency on
  running the offline solver. Regenerate only if scoring rules change.
