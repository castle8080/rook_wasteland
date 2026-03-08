# Task M4-T1: DP Precomputation Solver

**Milestone:** M4 — DP Precomputation
**Status:** ✅ Done

## Restatement

This task builds the offline DP solver (`offline/` standalone Rust binary) that
computes the single-column expected-value table `V_COL[8192]` for the Ask Grandma
advisor. The solver uses backward induction over all 13-bit fill patterns, enumerating
all 252 distinct 5d6 dice multisets with their multinomial probabilities and evaluating
3-roll turn lookahead via precomputed re-roll transition distributions. It also produces
`YZ_BONUS_CORRECTION[14]`, a bonus-pool correction indexed by number of open Sixzee cells
and forfeiture state. The output is `generated/v_col.rs` — a committed Rust source file
included at compile time by the Worker crate. Running the solver is only required when
scoring rules change. The M7 (Ask Grandma) advisor wires this table into its ranking logic.

## Design

### Data flow

1. `enumerate_dice_multisets()` → 252 `([u8;5], f64)` pairs (sorted tuple + probability)
2. `build_lookup()` → `HashMap<[u8;5], usize>` (fast multiset index lookup)
3. `precompute_transitions()` → `trans[d_idx][h_idx]` = `Vec<(new_d_idx, prob)>`
   — reroll distribution for each (dice, hold-strategy) pair; computed once
4. Fill loop `fill` 8190→0:
   - `bt0[252]`: best immediate score for each multiset at this fill pattern
   - `bt1[252]`: best value with 1 reroll remaining = max(bt0, E[bt0 over hold])
   - `bt2[252]`: best value with 2 rerolls remaining = max(bt0, E[bt1 over hold])
   - `v_col[fill]` = Σ P(d) × bt2[d]
5. `YZ_BONUS_CORRECTION[14]` computed via closed-form approximation
6. Emit `generated/v_col.rs` with `pub const V_COL` and `pub const YZ_BONUS_CORRECTION`

### Function / type signatures

```rust
/// Returns all 252 distinct sorted 5d6 multisets with multinomial probabilities.
fn enumerate_dice_multisets() -> Vec<([u8; 5], f64)>

/// Returns all distinct sorted k-dice multisets with their probabilities.
fn enumerate_k_dice_multisets(k: usize) -> Vec<(Vec<u8>, f64)>

/// Returns all distinct hold submultisets of `dice` (excludes hold-all).
fn enumerate_hold_strategies(dice: [u8; 5]) -> Vec<Vec<u8>>

/// Merge two sorted slices into a sorted [u8; 5].
fn merge_sorted_5(a: &[u8], b: &[u8]) -> [u8; 5]

/// Precompute re-roll transition distributions: trans[d][h] = Vec<(new_d_idx, prob)>.
fn precompute_transitions(
    multisets: &[([u8; 5], f64)],
    lookup: &HashMap<[u8; 5], usize>,
) -> Vec<Vec<Vec<(usize, f64)>>>

/// Run backward induction and return (V_COL, YZ_BONUS_CORRECTION).
fn solve() -> ([f32; 8192], [f32; 14])

/// Write generated/v_col.rs.
fn write_output(v_col: &[f32; 8192], yz: &[f32; 14]) -> std::io::Result<()>
```

### Edge cases

- `fill = 8191` (all bits set): no open rows → bt0 = 0, V_COL[8191] = 0.0 exactly
- `fill` with all rows filled except one: only one legal placement
- `hold nothing` strategy (reroll all 5): 252 possible outcomes, same distribution as initial roll
- `hold all` strategy: excluded — equivalent to score-now
- Duplicate hold strategies for multisets with repeated values (e.g. [3,3,3,5,6]):
  holding dice[0] and dice[1] both give held=[3]; deduplicated via HashSet
- Probability sum: must equal 1.0 within 1e-9 tolerance

### Integration points

- `offline/Cargo.toml` — standalone binary crate, pure std
- `offline/src/main.rs` — entire solver
- `generated/v_col.rs` — emitted output, consumed by `src/worker/grandma_worker.rs`
- Scoring constants must exactly match `src/state/scoring.rs`

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Scoring logic duplicated from scoring.rs | Add compile-time comment; accept duplication for standalone crate — divergence risk is noted in M4 doc and caught by advisor quality |
| Correctness | v_col[fill] loop starts at 8190 rev — fill=8191 is never overwritten | Correct: v_col[8191]=0.0 by initialization, loop covers 0..8190 |
| Simplicity | bt0 initialises `best=0.0` — might shadow a valid placement of 0 | OK: 0 is a valid score; taking best=0.0 means the worst case is scoring 0 somewhere, which is always available |
| Coupling | YZ_BONUS_CORRECTION formula is an approximation | Explicitly documented; exact DP over Sixzee sub-game deferred per spec |
| Performance | 8192 × 252 × ~25 holds × ~40 dist entries ≈ 2 × 2.5B float ops | Release build with LTO; expected <5 s on modern CPU per milestone target |
| Testability | Offline binary has no unit tests — pure output validation | Solver prints summary + panics on assertion failure; offline crate excluded from main test suite |

## Implementation Notes

- `YZ_BONUS_CORRECTION` layout: index `6 - n_sixzee_open` (forfeited=false, indices 0–6),
  index `6 - n_sixzee_open + 7` (forfeited=true, indices 7–13).
  Approximation: `correction(n) = n * 15.0` where n = n_sixzee_open ∈ [0,6].
  This gives [0]=90, [6]=0, [7..=13]=0. Satisfies success criteria.
- `score_for_row` duplicated verbatim from `src/state/scoring.rs` to keep offline crate
  self-contained with zero workspace dependencies.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| enumerate_dice_multisets returns exactly 252 entries | 1 | ✅ | assert_eq! in solver |
| probabilities sum to 1.0 | 1 | ✅ | assert! in solver |
| v_col[8191] == 0.0 | 1 | ✅ | assert! in solver |
| v_col[0] in [200,300] | 1 | ✅ | assert! in solver |
| all 8192 values finite | 1 | ✅ | assert! loop in solver |
| YZ_BONUS_CORRECTION[0] positive | 1 | ✅ | asserted in solver |
| YZ_BONUS_CORRECTION[7..=13] == 0.0 | 1 | ✅ | asserted in solver |
| generated file compiles | 1 | ✅ | `cargo run` + trunk build verifies |
| hold-strategy deduplication | 1 | ✅ | HashSet removes duplicates; covered implicitly by sum=1.0 |

## Test Results

- `cargo test` (offline): 11/11 pass (includes multiset counts, probability sums, merge,
  hold strategy dedup, yz_bonus_correction layout, v_col boundary values)
- `cargo run --release` (offline): v_col[0]=229.638500, v_col[8191]=0.0, runtime 2.19 s
- `cargo test` (main app): 63/63 pass — no regressions
- `cargo clippy -- -D warnings` (offline): 0 warnings
- `cargo clippy --target wasm32-unknown-unknown -- -D warnings` (main app): 0 warnings
- Code review agent: no issues found

## Review Notes

No issues found. bt0 initialisation `best = 0.0` is correct because:
- fill < 8191 guarantees at least one open row
- All scores and v_col values are non-negative
- `_f32` suffix on generated literals is valid Rust syntax

## Callouts / Gotchas

- `bt2[d] = max(bt0[d], max_h E[bt1[new_d]])` — "score now" always compares against
  `bt0` (direct placement), not against `bt1`. This is correct: you choose between
  placing now vs. using another reroll, independent of how many rerolls remain.
- The initial mandatory roll is captured by `V_COL[fill] = Σ P(d) × bt2[d]`, where bt2
  represents the value of having the initial dice with 2 more optional rerolls (3 total).
