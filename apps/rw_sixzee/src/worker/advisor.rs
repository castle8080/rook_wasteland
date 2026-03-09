//! Ask Grandma computation engine.
//!
//! Pure Rust — no web_sys or Leptos.  Compiles for both native (tests) and
//! wasm32 targets.
//!
//! # Algorithm
//!
//! Two categories of candidate actions are generated:
//!
//! **Score-now:** for every open `(col, row)` cell, estimate the final score
//! as `already_scored + Σ V_COL[fill] + marginal + bonus_pool + bonus_correction`.
//!
//! **Reroll** (when `rolls_used < 3`): generate all 32 hold-masks, deduplicate
//! by sorted held-value tuple, then for each unique strategy simulate the
//! reroll outcomes (exact enumeration when `k ≤ 3`; 300 Monte Carlo samples
//! when `k ≥ 4`) and average the best-score-now over those outcomes.
//!
//! All candidates are sorted by `est_final_score` descending; the top 5 are
//! returned.

use rand::Rng;

use crate::state::scoring::{score_for_row, ROW_COUNT, ROW_LABELS, ROW_SIXZEE};
use crate::worker::messages::{ActionKind, GrandmaAction, GrandmaRequest, GrandmaResponse};

// ─── DP tables (generated) ───────────────────────────────────────────────────

// Wrap the include! in a submodule so inner #![allow] attributes can suppress
// lints that fire on the generated large const arrays and high-precision floats.
mod dp_tables {
    #![allow(clippy::large_const_arrays, clippy::excessive_precision)]
    // Include the auto-generated V_COL and YZ_BONUS_CORRECTION arrays.
    // Permitted include! / expect site per tech spec §15.
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/v_col.rs"));
}
use dp_tables::{V_COL, YZ_BONUS_CORRECTION};

// ─── Number of MC samples for k ≥ 4 unheld dice ─────────────────────────────

/// Number of Monte Carlo samples when k ≥ 4 (6^4 = 1296 exact; 6^5 = 7776 exact
/// is fine for probabilities but we cap the main EV loop for latency).
const MC_SAMPLES: usize = 300;

// ─── Public entry point ───────────────────────────────────────────────────────

/// Compute the top-5 recommended actions for the given game state.
pub fn compute_grandma_actions(req: &GrandmaRequest) -> GrandmaResponse {
    let base = BaseEstimate::from_request(req);

    let mut candidates: Vec<GrandmaAction> = Vec::with_capacity(64);

    // ── Score-now candidates ──────────────────────────────────────────────────
    for col in 0..6usize {
        for (row, _) in ROW_LABELS.iter().enumerate() {
            if req.cells[col][row].is_some() {
                continue; // already filled
            }
            let points = score_for_row(row, req.dice);
            let marginal = marginal_score_now(&base, col, row, points, req);
            let est = (base.already_scored as f32
                + base.sum_vcol
                + marginal
                + req.bonus_pool as f32
                + base.bonus_correction)
                .max(0.0) as u32;

            let description = format!(
                "Score {} → Column {}  ({} pts)",
                ROW_LABELS[row],
                col + 1,
                points
            );
            let detail = format!("{points} pts");

            candidates.push(GrandmaAction {
                kind: ActionKind::Score { col, row, points },
                description,
                detail,
                est_final_score: est,
            });
        }
    }

    // ── Reroll candidates ─────────────────────────────────────────────────────
    if req.rolls_used < 3 {
        let reroll_candidates = generate_reroll_candidates(req, &base);
        candidates.extend(reroll_candidates);
    }

    // ── Sort and take top 5 ───────────────────────────────────────────────────
    candidates.sort_by(|a, b| b.est_final_score.cmp(&a.est_final_score));
    candidates.truncate(5);

    GrandmaResponse { actions: candidates }
}

// ─── Base estimate ────────────────────────────────────────────────────────────

/// Pre-computed values shared across all candidate evaluations.
struct BaseEstimate {
    /// Sum of all scored (Some) cell values across the entire board.
    already_scored: u32,
    /// `Σ V_COL[fill_pattern(col)]` for all 6 columns.
    sum_vcol: f32,
    /// `V_COL[fill_pattern(col)]` per column.
    fill_vcol: [f32; 6],
    /// Bit-mask of filled rows per column: `fills[col] = Σ (1<<row) for filled rows`.
    fills: [usize; 6],
    /// Bonus pool correction from YZ_BONUS_CORRECTION.
    bonus_correction: f32,
}

impl BaseEstimate {
    fn from_request(req: &GrandmaRequest) -> Self {
        let mut already_scored = 0u32;
        let mut fills = [0usize; 6];
        let mut fill_vcol = [0f32; 6];

        for col in 0..6 {
            for row in 0..ROW_COUNT {
                if let Some(v) = req.cells[col][row] {
                    already_scored += v as u32;
                    fills[col] |= 1 << row;
                }
            }
            fill_vcol[col] = V_COL[fills[col]];
        }

        let sum_vcol: f32 = fill_vcol.iter().sum();

        let n_sixzee_open = (0..6)
            .filter(|&col| req.cells[col][ROW_SIXZEE].is_none())
            .count();
        let correction_idx = if req.bonus_forfeited {
            (6 - n_sixzee_open) + 7
        } else {
            6 - n_sixzee_open
        };
        let bonus_correction = YZ_BONUS_CORRECTION[correction_idx];

        BaseEstimate {
            already_scored,
            sum_vcol,
            fill_vcol,
            fills,
            bonus_correction,
        }
    }
}

// ─── Score-now marginal ────────────────────────────────────────────────────────

/// Marginal change to `sum_vcol` plus the points from placing `(col, row)`.
fn marginal_score_now(
    base: &BaseEstimate,
    col: usize,
    row: usize,
    points: u8,
    req: &GrandmaRequest,
) -> f32 {
    let new_fill = base.fills[col] | (1 << row);
    let vcol_delta = V_COL[new_fill] - base.fill_vcol[col];

    // If scratching a Sixzee cell (points = 0), the bonus is forfeited.
    // Recompute bonus correction for the post-placement state.
    let extra_correction = if row == ROW_SIXZEE && points == 0 && !req.bonus_forfeited {
        let n_open_after = (0..6)
            .filter(|&c| c != col && req.cells[c][ROW_SIXZEE].is_none())
            .count();
        // After forfeit: correction_idx = (6 - n_open_after) + 7
        let new_idx = (6 - n_open_after) + 7;
        YZ_BONUS_CORRECTION[new_idx] - base.bonus_correction
    } else {
        0.0
    };

    points as f32 + vcol_delta + extra_correction
}

// ─── Best score-now for a given dice outcome ─────────────────────────────────

/// For a reroll simulation outcome, find the best marginal score-now value.
fn best_marginal_for_dice(base: &BaseEstimate, cells: &[[Option<u8>; ROW_COUNT]; 6], dice: [u8; 5]) -> f32 {
    let mut best = f32::NEG_INFINITY;
    for (col, col_cells) in cells.iter().enumerate() {
        for (row, cell) in col_cells.iter().enumerate() {
            if cell.is_some() {
                continue;
            }
            let pts = score_for_row(row, dice);
            let new_fill = base.fills[col] | (1 << row);
            let val = pts as f32 + V_COL[new_fill] - base.fill_vcol[col];
            if val > best {
                best = val;
            }
        }
    }
    if best == f32::NEG_INFINITY {
        0.0
    } else {
        best
    }
}

// ─── Reroll candidates ────────────────────────────────────────────────────────

/// Generate all unique reroll strategies and score each via simulation.
fn generate_reroll_candidates(req: &GrandmaRequest, base: &BaseEstimate) -> Vec<GrandmaAction> {
    // Enumerate all 32 hold masks, deduplicate by sorted held-value tuple.
    let mut seen: Vec<[u8; 5]> = Vec::with_capacity(32);
    let mut unique_masks: Vec<[bool; 5]> = Vec::with_capacity(32);

    for mask_bits in 0u8..32 {
        let hold_mask: [bool; 5] = core::array::from_fn(|i| (mask_bits >> i) & 1 == 1);
        let key = sorted_held_values(req.dice, hold_mask);
        if !seen.contains(&key) {
            seen.push(key);
            unique_masks.push(hold_mask);
        }
    }

    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(unique_masks.len());

    for hold_mask in unique_masks {
        let n_unheld = hold_mask.iter().filter(|&&h| !h).count();

        // Hold-all (n_unheld == 0) is not a reroll — skip.
        if n_unheld == 0 {
            continue;
        }

        // ── Compute expected marginal via simulation ──────────────────────────
        let ev = if n_unheld <= 3 {
            // Exact enumeration (6^1=6, 6^2=36, 6^3=216 outcomes).
            exact_ev(req, base, hold_mask, n_unheld)
        } else {
            // Monte Carlo (6^4=1296, 6^5=7776 — sample for speed).
            mc_ev(req, base, hold_mask, n_unheld, &mut rng)
        };

        let est = (base.already_scored as f32
            + base.sum_vcol
            + ev
            + req.bonus_pool as f32
            + base.bonus_correction)
            .max(0.0) as u32;

        // ── Description ───────────────────────────────────────────────────────
        let held_vals: Vec<u8> = req
            .dice
            .iter()
            .zip(hold_mask.iter())
            .filter_map(|(&d, &h)| if h { Some(d) } else { None })
            .collect();

        let description = if held_vals.is_empty() {
            format!("Hold nothing — reroll all {n_unheld} dice")
        } else {
            let held_str: Vec<String> = held_vals.iter().map(|v| v.to_string()).collect();
            format!("Hold [{}] — reroll {n_unheld} dice", held_str.join(", "))
        };

        // ── Probability detail (exact enumeration over all 6^k outcomes) ─────
        let detail = compute_prob_detail(req.dice, hold_mask, n_unheld);

        result.push(GrandmaAction {
            kind: ActionKind::Reroll { hold_mask },
            description,
            detail,
            est_final_score: est,
        });
    }

    result
}

// ─── Exact EV enumeration ─────────────────────────────────────────────────────

/// Exact expected marginal for k ≤ 3 unheld dice.
fn exact_ev(
    req: &GrandmaRequest,
    base: &BaseEstimate,
    hold_mask: [bool; 5],
    n_unheld: usize,
) -> f32 {
    let total = 6usize.pow(n_unheld as u32);
    let mut sum = 0.0f32;

    for outcome_idx in 0..total {
        let outcome_dice = make_outcome_dice(req.dice, hold_mask, outcome_idx, n_unheld);
        sum += best_marginal_for_dice(base, &req.cells, outcome_dice);
    }

    sum / total as f32
}

// ─── Monte Carlo EV ───────────────────────────────────────────────────────────

/// Monte Carlo expected marginal for k ≥ 4 unheld dice.
fn mc_ev<R: Rng>(
    req: &GrandmaRequest,
    base: &BaseEstimate,
    hold_mask: [bool; 5],
    _n_unheld: usize,
    rng: &mut R,
) -> f32 {
    let mut sum = 0.0f32;

    for _ in 0..MC_SAMPLES {
        let mut dice = req.dice;
        for i in 0..5 {
            if !hold_mask[i] {
                dice[i] = rng.gen_range(1u8..=6);
            }
        }
        sum += best_marginal_for_dice(base, &req.cells, dice);
    }

    sum / MC_SAMPLES as f32
}

// ─── Probability detail ───────────────────────────────────────────────────────

/// Enumerate all 6^k outcomes exactly and compute probabilities for notable combos.
fn compute_prob_detail(dice: [u8; 5], hold_mask: [bool; 5], n_unheld: usize) -> String {
    let total = 6usize.pow(n_unheld as u32);
    let mut count_sixzee = 0usize;
    let mut count_4k = 0usize;
    let mut count_fh = 0usize;
    let mut count_lg = 0usize;
    let mut count_3k = 0usize;

    for outcome_idx in 0..total {
        let d = make_outcome_dice(dice, hold_mask, outcome_idx, n_unheld);
        let counts = value_counts(d);
        let max_count = *counts.iter().max().unwrap_or(&0);

        if max_count == 5 {
            count_sixzee += 1;
        }
        if max_count >= 4 {
            count_4k += 1;
        }
        let has_three = counts.contains(&3);
        let has_two = counts.contains(&2);
        if has_three && has_two {
            count_fh += 1;
        }
        if crate::state::scoring::score_large_straight(d) > 0 {
            count_lg += 1;
        }
        if max_count >= 3 {
            count_3k += 1;
        }
    }

    let mut parts: Vec<String> = Vec::new();

    let pct = |c: usize| -> u32 { (c * 100 / total) as u32 };

    if count_sixzee > 0 {
        parts.push(format!("Sixzee: ~{}%", pct(count_sixzee)));
    }
    if count_4k > 0 && count_4k != count_sixzee {
        parts.push(format!("4-of-a-kind: ~{}%", pct(count_4k)));
    }
    if count_fh > 0 {
        parts.push(format!("Full House: ~{}%", pct(count_fh)));
    }
    if count_lg > 0 {
        parts.push(format!("Lg. Straight: ~{}%", pct(count_lg)));
    }
    if count_3k > 0 && parts.len() < 3 {
        parts.push(format!("3-of-a-kind: ~{}%", pct(count_3k)));
    }

    if parts.is_empty() {
        "Various outcomes".to_string()
    } else {
        parts.join("   ")
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Expand `outcome_idx` (base-6) into unheld dice slots, returning the full 5-die array.
fn make_outcome_dice(
    base_dice: [u8; 5],
    hold_mask: [bool; 5],
    outcome_idx: usize,
    n_unheld: usize,
) -> [u8; 5] {
    let mut dice = base_dice;
    let mut idx = outcome_idx;
    let mut slot = 0usize;

    for i in 0..5 {
        if !hold_mask[i] && slot < n_unheld {
            dice[i] = (idx % 6) as u8 + 1;
            idx /= 6;
            slot += 1;
        }
    }

    dice
}

/// Returns the sorted tuple of held die values (used for deduplication).
fn sorted_held_values(dice: [u8; 5], hold_mask: [bool; 5]) -> [u8; 5] {
    let mut vals = [0u8; 5];
    let mut count = 0usize;
    for i in 0..5 {
        if hold_mask[i] {
            vals[count] = dice[i];
            count += 1;
        }
    }
    // Sort only the occupied slots; zeros sort to front, held values follow.
    vals[..count].sort_unstable();
    // Place sorted held values at the end so zeros come first (canonical form).
    let mut key = [0u8; 5];
    key[(5 - count)..].copy_from_slice(&vals[..count]);
    key
}

/// Per-value counts (index 0 = value 1).
fn value_counts(dice: [u8; 5]) -> [u8; 6] {
    let mut counts = [0u8; 6];
    for &d in &dice {
        if (1..=6).contains(&d) {
            counts[(d - 1) as usize] += 1;
        }
    }
    counts
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker::messages::GrandmaRequest;

    fn fresh_request(dice: [u8; 5]) -> GrandmaRequest {
        GrandmaRequest {
            cells: [[None; 13]; 6],
            dice,
            held: [false; 5],
            rolls_used: 1,
            bonus_pool: 0,
            bonus_forfeited: false,
        }
    }

    #[test]
    fn returns_five_actions_on_fresh_board() {
        let req = fresh_request([3, 3, 3, 2, 5]);
        let resp = compute_grandma_actions(&req);
        assert_eq!(resp.actions.len(), 5, "expected exactly 5 actions");
    }

    #[test]
    fn actions_sorted_descending() {
        let req = fresh_request([1, 2, 3, 4, 5]);
        let resp = compute_grandma_actions(&req);
        for window in resp.actions.windows(2) {
            assert!(
                window[0].est_final_score >= window[1].est_final_score,
                "actions must be sorted descending by est_final_score"
            );
        }
    }

    #[test]
    fn five_identical_dice_reroll_dedup_yields_five_strategies() {
        // [5,5,5,5,5]: hold 0, 1, 2, 3, or 4 dice → 5 unique strategies.
        // Plus score-now candidates → final list is top-5 which may include rerolls.
        let req = fresh_request([5, 5, 5, 5, 5]);
        let base = BaseEstimate::from_request(&req);
        let rerolls = generate_reroll_candidates(&req, &base);
        // Hold-0 through hold-4 = 5 unique reroll strategies.
        assert_eq!(
            rerolls.len(),
            5,
            "5 unique reroll strategies for 5 identical dice, got: {rerolls:?}"
        );
    }

    #[test]
    fn dp_sanity_all_filled_is_zero() {
        // V_COL[0b1_1111_1111_1111] = 0.0 (all 13 rows filled, no more value)
        assert_eq!(
            V_COL[0b1_1111_1111_1111],
            0.0,
            "V_COL[8191] must be 0.0"
        );
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn dp_sanity_empty_board_in_expected_range() {
        // V_COL[0] = expected score from a fresh column, should be ~200–300.
        assert!(
            V_COL[0] > 200.0 && V_COL[0] < 300.0,
            "V_COL[0] = {} expected in [200, 300]",
            V_COL[0]
        );
    }

    #[test]
    fn sorted_held_values_dedup() {
        // Two masks that hold the same values should produce the same key.
        let dice = [5u8, 3, 5, 2, 5];
        let mask_a = [true, false, true, false, false]; // holds 5, 5
        let mask_b = [false, false, true, false, true]; // holds 5, 5
        assert_eq!(
            sorted_held_values(dice, mask_a),
            sorted_held_values(dice, mask_b),
            "same held values must produce identical dedup key"
        );
    }

    #[test]
    fn score_now_on_nearly_full_board_returns_five() {
        // Only 5 cells open — must still return 5 actions (all score-now, no reroll).
        let mut cells = [[Some(0u8); 13]; 6];
        // Open exactly 5 cells across different columns.
        cells[0][0] = None;
        cells[1][1] = None;
        cells[2][2] = None;
        cells[3][3] = None;
        cells[4][4] = None;
        let req = GrandmaRequest {
            cells,
            dice: [1, 2, 3, 4, 5],
            held: [false; 5],
            rolls_used: 1,
            bonus_pool: 0,
            bonus_forfeited: false,
        };
        let resp = compute_grandma_actions(&req);
        assert_eq!(resp.actions.len(), 5);
    }
}
