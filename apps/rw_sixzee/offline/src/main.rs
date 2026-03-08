//! Offline DP precomputation solver for rw_sixzee.
//!
//! Computes the single-column expected-value table `V_COL[8192]` via backward
//! induction over all 8,192 fill patterns (13-bit bitmasks where bit i = 1 means
//! row i is already filled). Also produces `YZ_BONUS_CORRECTION[14]`, an additive
//! correction for the Sixzee bonus pool, indexed by open Sixzee cells and forfeiture.
//!
//! Output: `../generated/v_col.rs` — committed to the repo and included at compile
//! time by the Worker crate.
//!
//! Run: `cargo run --release` from the `offline/` directory.

use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

// ── Row index constants (must stay in sync with src/state/scoring.rs) ────────

const ROW_ONES: usize = 0;
const ROW_TWOS: usize = 1;
const ROW_THREES: usize = 2;
const ROW_FOURS: usize = 3;
const ROW_FIVES: usize = 4;
const ROW_SIXES: usize = 5;
const ROW_THREE_OF_A_KIND: usize = 6;
const ROW_FOUR_OF_A_KIND: usize = 7;
const ROW_FULL_HOUSE: usize = 8;
const ROW_SMALL_STRAIGHT: usize = 9;
const ROW_LARGE_STRAIGHT: usize = 10;
const ROW_SIXZEE: usize = 11;
const ROW_CHANCE: usize = 12;
const ROW_COUNT: usize = 13;

/// Total fill patterns (2^13 = 8192 bitmasks over 13 rows).
const FILL_COUNT: usize = 8192;
/// Pattern with all 13 rows filled.
const FILL_ALL: usize = FILL_COUNT - 1; // 0b1_1111_1111_1111

// ── Scoring functions (duplicated from src/state/scoring.rs) ─────────────────
//
// Keeping the offline crate self-contained avoids any workspace dependency on
// the WASM library. If scoring rules change, update both files together and
// re-run the solver.

fn factorial(n: u64) -> u64 {
    (1..=n).product()
}

fn has_n_of_a_kind(dice: &[u8; 5], n: u8) -> bool {
    value_counts(dice).iter().any(|&c| c >= n)
}

fn value_counts(dice: &[u8; 5]) -> [u8; 6] {
    let mut counts = [0u8; 6];
    for &d in dice {
        if (1..=6).contains(&d) {
            counts[(d - 1) as usize] += 1;
        }
    }
    counts
}

fn unique_sorted(dice: &[u8; 5]) -> Vec<u8> {
    let mut v: Vec<u8> = dice.to_vec();
    v.sort_unstable();
    v.dedup();
    v
}

fn score_for_row(row: usize, dice: [u8; 5]) -> u8 {
    match row {
        ROW_ONES => dice.iter().filter(|&&d| d == 1).sum(),
        ROW_TWOS => dice.iter().filter(|&&d| d == 2).sum(),
        ROW_THREES => dice.iter().filter(|&&d| d == 3).sum(),
        ROW_FOURS => dice.iter().filter(|&&d| d == 4).sum(),
        ROW_FIVES => dice.iter().filter(|&&d| d == 5).sum(),
        ROW_SIXES => dice.iter().filter(|&&d| d == 6).sum(),
        ROW_THREE_OF_A_KIND => {
            if has_n_of_a_kind(&dice, 3) {
                dice.iter().sum()
            } else {
                0
            }
        }
        ROW_FOUR_OF_A_KIND => {
            if has_n_of_a_kind(&dice, 4) {
                dice.iter().sum()
            } else {
                0
            }
        }
        ROW_FULL_HOUSE => {
            let counts = value_counts(&dice);
            if counts.contains(&3) && counts.contains(&2) {
                25
            } else {
                0
            }
        }
        ROW_SMALL_STRAIGHT => {
            let unique = unique_sorted(&dice);
            for start in 1u8..=3 {
                if (start..start + 4).all(|v| unique.contains(&v)) {
                    return 30;
                }
            }
            0
        }
        ROW_LARGE_STRAIGHT => {
            let unique = unique_sorted(&dice);
            if unique.len() == 5
                && (unique == vec![1, 2, 3, 4, 5] || unique == vec![2, 3, 4, 5, 6])
            {
                40
            } else {
                0
            }
        }
        ROW_SIXZEE => {
            if dice.iter().all(|&d| d == dice[0]) {
                50
            } else {
                0
            }
        }
        ROW_CHANCE => dice.iter().sum(),
        _ => panic!("score_for_row: row {row} out of range 0–12"),
    }
}

// ── Dice multiset enumeration ─────────────────────────────────────────────────

/// Returns all 252 distinct sorted 5d6 multisets with their multinomial probabilities.
///
/// Each entry is `(sorted_dice, P(sorted_dice))` where P sums to 1.0 over all entries.
fn enumerate_dice_multisets() -> Vec<([u8; 5], f64)> {
    let total = 6u64.pow(5) as f64; // 7776
    let mut result = Vec::with_capacity(252);
    for v1 in 1u8..=6 {
        for v2 in v1..=6 {
            for v3 in v2..=6 {
                for v4 in v3..=6 {
                    for v5 in v4..=6 {
                        let dice = [v1, v2, v3, v4, v5];
                        let mut counts = [0u64; 6];
                        for &d in &dice {
                            counts[(d - 1) as usize] += 1;
                        }
                        // multinomial coefficient = 5! / (n1! * n2! * ... * n6!)
                        let denom: u64 = counts.iter().map(|&c| factorial(c)).product();
                        let coeff = factorial(5) / denom;
                        result.push((dice, coeff as f64 / total));
                    }
                }
            }
        }
    }
    result
}

/// Returns all distinct sorted k-dice multisets with their multinomial probabilities.
///
/// k=0 returns a single entry `([], 1.0)` representing "no dice to roll".
fn enumerate_k_dice_multisets(k: usize) -> Vec<(Vec<u8>, f64)> {
    if k == 0 {
        return vec![(vec![], 1.0)];
    }
    let total = 6u64.pow(k as u32) as f64;
    let mut result = Vec::new();

    // Iterate over all sorted k-tuples of values 1–6 (combinations with replacement).
    let mut current = vec![1u8; k];
    loop {
        let mut counts = [0u64; 6];
        for &d in &current {
            counts[(d - 1) as usize] += 1;
        }
        let denom: u64 = counts.iter().map(|&c| factorial(c)).product();
        let prob = factorial(k as u64) as f64 / denom as f64 / total;
        result.push((current.clone(), prob));

        // Advance to the next non-decreasing tuple.
        let mut pos = k - 1;
        loop {
            if current[pos] < 6 {
                current[pos] += 1;
                // Reset all later positions to the same value (maintain sorted order).
                for j in (pos + 1)..k {
                    current[j] = current[pos];
                }
                break;
            }
            if pos == 0 {
                return result;
            }
            pos -= 1;
        }
    }
}

// ── Hold strategy enumeration ─────────────────────────────────────────────────

/// Returns all distinct hold submultisets of `dice` as sorted `Vec<u8>`.
///
/// Excludes the full hold (all 5 dice held, no reroll) because that is dominated
/// by simply scoring now. The empty hold (reroll all 5) is included.
fn enumerate_hold_strategies(dice: [u8; 5]) -> Vec<Vec<u8>> {
    let mut seen = std::collections::HashSet::new();
    for mask in 0u8..32u8 {
        if mask == 31 {
            continue; // skip hold-all
        }
        let mut held: Vec<u8> = (0usize..5)
            .filter(|&i| mask & (1 << i) != 0)
            .map(|i| dice[i])
            .collect();
        held.sort_unstable();
        seen.insert(held);
    }
    seen.into_iter().collect()
}

// ── Sorted merge ──────────────────────────────────────────────────────────────

/// Merge two sorted slices into a sorted `[u8; 5]`.
fn merge_sorted_5(a: &[u8], b: &[u8]) -> [u8; 5] {
    debug_assert_eq!(a.len() + b.len(), 5, "held + rerolled must total 5 dice");
    let mut out = [0u8; 5];
    let (mut ai, mut bi, mut oi) = (0, 0, 0);
    while ai < a.len() && bi < b.len() {
        if a[ai] <= b[bi] {
            out[oi] = a[ai];
            ai += 1;
        } else {
            out[oi] = b[bi];
            bi += 1;
        }
        oi += 1;
    }
    while ai < a.len() {
        out[oi] = a[ai];
        ai += 1;
        oi += 1;
    }
    while bi < b.len() {
        out[oi] = b[bi];
        bi += 1;
        oi += 1;
    }
    out
}

// ── Transition table ──────────────────────────────────────────────────────────

/// Build the lookup map from sorted 5-tuple to its index in `multisets`.
fn build_lookup(multisets: &[([u8; 5], f64)]) -> HashMap<[u8; 5], usize> {
    multisets
        .iter()
        .enumerate()
        .map(|(i, &(d, _))| (d, i))
        .collect()
}

/// Precompute re-roll transition distributions.
///
/// `trans[d_idx][h_idx]` = compact probability distribution over result dice indices
/// when holding `holds[d_idx][h_idx]` and re-rolling the remaining dice.
///
/// This is computed once and reused for all 8,192 fill patterns.
fn precompute_transitions(
    multisets: &[([u8; 5], f64)],
    lookup: &HashMap<[u8; 5], usize>,
) -> Vec<Vec<Vec<(usize, f64)>>> {
    let mut trans = Vec::with_capacity(multisets.len());
    for &(dice, _) in multisets {
        let holds = enumerate_hold_strategies(dice);
        let mut dice_trans: Vec<Vec<(usize, f64)>> = Vec::with_capacity(holds.len());
        for held in &holds {
            let k = 5 - held.len();
            let rerolls = enumerate_k_dice_multisets(k);
            // Aggregate probability into a compact (new_idx → prob) map.
            let mut dist: HashMap<usize, f64> = HashMap::new();
            for (reroll, prob) in &rerolls {
                let new_dice = merge_sorted_5(held, reroll);
                let new_idx = *lookup
                    .get(&new_dice)
                    .expect("all 5-dice sorted tuples must be in the lookup table");
                *dist.entry(new_idx).or_insert(0.0) += prob;
            }
            dice_trans.push(dist.into_iter().collect());
        }
        trans.push(dice_trans);
    }
    trans
}

// ── DP Backward Induction ─────────────────────────────────────────────────────

/// Run the full backward-induction DP and return `(V_COL, YZ_BONUS_CORRECTION)`.
fn solve() -> ([f32; FILL_COUNT], [f32; 14]) {
    let multisets = enumerate_dice_multisets();
    assert_eq!(multisets.len(), 252, "must have exactly 252 distinct 5d6 multisets");
    let prob_sum: f64 = multisets.iter().map(|(_, p)| p).sum();
    assert!(
        (prob_sum - 1.0).abs() < 1e-9,
        "dice probabilities must sum to 1.0, got {prob_sum}"
    );

    let lookup = build_lookup(&multisets);
    let trans = precompute_transitions(&multisets, &lookup);

    // v_col[FILL_ALL] = 0.0 (no rows left to score); all other entries computed below.
    let mut v_col = [0.0f64; FILL_COUNT];

    // Iterate fill patterns from FILL_ALL-1 down to 0 (backward induction).
    for fill in (0..FILL_ALL).rev() {
        // ── bt0: must score immediately with current dice ─────────────────────
        // bt0[d] = max over open rows of (score(row, dice) + V_col[fill | (1<<row)])
        let mut bt0 = [0.0f64; 252];
        for (d_idx, &(dice, _)) in multisets.iter().enumerate() {
            let mut best = 0.0f64;
            for row in 0..ROW_COUNT {
                if fill & (1 << row) == 0 {
                    // row is open
                    let score = score_for_row(row, dice) as f64;
                    let future = v_col[fill | (1 << row)];
                    let val = score + future;
                    if val > best {
                        best = val;
                    }
                }
            }
            bt0[d_idx] = best;
        }

        // ── bt1: one reroll remaining ─────────────────────────────────────────
        // bt1[d] = max(bt0[d],  max_h  Σ p(h→d') · bt0[d'])
        let mut bt1 = [0.0f64; 252];
        for (d_idx, _) in multisets.iter().enumerate() {
            let mut best = bt0[d_idx]; // score now
            for hold_dist in &trans[d_idx] {
                let expected: f64 = hold_dist
                    .iter()
                    .map(|&(new_idx, prob)| prob * bt0[new_idx])
                    .sum();
                if expected > best {
                    best = expected;
                }
            }
            bt1[d_idx] = best;
        }

        // ── bt2: two rerolls remaining ────────────────────────────────────────
        // bt2[d] = max(bt0[d],  max_h  Σ p(h→d') · bt1[d'])
        let mut bt2 = [0.0f64; 252];
        for (d_idx, _) in multisets.iter().enumerate() {
            let mut best = bt0[d_idx]; // score now
            for hold_dist in &trans[d_idx] {
                let expected: f64 = hold_dist
                    .iter()
                    .map(|&(new_idx, prob)| prob * bt1[new_idx])
                    .sum();
                if expected > best {
                    best = expected;
                }
            }
            bt2[d_idx] = best;
        }

        // V_col[fill] = expected value of bt2 over the initial mandatory roll.
        v_col[fill] = multisets
            .iter()
            .enumerate()
            .map(|(d_idx, &(_, prob))| prob * bt2[d_idx])
            .sum();
    }

    // ── Validation ────────────────────────────────────────────────────────────
    assert!(
        v_col[FILL_ALL] == 0.0,
        "v_col[8191] must be exactly 0.0 (all rows filled)"
    );
    assert!(
        (200.0..=300.0).contains(&v_col[0]),
        "v_col[0] = {} is outside expected range [200, 300]",
        v_col[0]
    );
    for (i, &v) in v_col.iter().enumerate() {
        assert!(v.is_finite(), "v_col[{i}] is not finite: {v}");
    }

    println!("v_col[0]    = {:.6}  (expected ~230–280)", v_col[0]);
    println!("v_col[8191] = {:.6}  (must be 0.0)", v_col[8191]);
    let (min_v, max_v) = v_col
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(v), hi.max(v))
        });
    println!("v_col min   = {min_v:.6}");
    println!("v_col max   = {max_v:.6}");

    // ── YZ_BONUS_CORRECTION ───────────────────────────────────────────────────
    //
    // Layout (14 entries):
    //   Index `6 - n`      (n = n_sixzee_open, 0..=6): forfeited=false
    //   Index `6 - n + 7`  (n = n_sixzee_open, 0..=6): forfeited=true = 0.0
    //
    //   [0] = n=6 open, not forfeited → maximum correction (all 6 cells available)
    //   [6] = n=0 open, not forfeited → 0.0 (all filled; no marginal correction)
    //   [7..=13]            = 0.0     (bonus forfeited; no future bonus possible)
    //
    // Approximation: correction(n) = n × 15.0
    // (each open Sixzee cell contributes ~15 expected points toward the bonus pool)
    let mut yz = [0.0f32; 14];
    for n in 0usize..=6 {
        let not_forfeited_idx = 6 - n;
        yz[not_forfeited_idx] = (n as f32) * 15.0;
        // forfeited entries remain 0.0
    }

    assert!(
        yz[0] > 0.0,
        "YZ_BONUS_CORRECTION[0] must be positive (n=6 open)"
    );
    for (i, &v) in yz.iter().enumerate().skip(7) {
        assert!(v == 0.0, "YZ_BONUS_CORRECTION[{i}] must be 0.0 (forfeited)");
    }
    println!(
        "YZ_BONUS_CORRECTION[0]  = {} (n=6 open, not forfeited)",
        yz[0]
    );
    println!(
        "YZ_BONUS_CORRECTION[6]  = {} (n=0 open, not forfeited)",
        yz[6]
    );
    println!("YZ_BONUS_CORRECTION[7]  = {} (forfeited)", yz[7]);

    // Cast v_col to f32 for output (f64 used throughout for precision).
    let mut v_col_f32 = [0.0f32; FILL_COUNT];
    for (i, &v) in v_col.iter().enumerate() {
        v_col_f32[i] = v as f32;
    }

    (v_col_f32, yz)
}

// ── Code generation ───────────────────────────────────────────────────────────

/// Write `../generated/v_col.rs` containing the two pub const arrays.
fn write_output(v_col: &[f32; FILL_COUNT], yz: &[f32; 14]) -> std::io::Result<()> {
    let out_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../generated/v_col.rs");

    let mut buf = String::new();

    writeln!(
        buf,
        "// Auto-generated by apps/rw_sixzee/offline/src/main.rs — do not edit by hand.\n\
         // Re-run `cargo run --release` from apps/rw_sixzee/offline/ to regenerate.\n"
    )
    .unwrap();

    // V_COL
    writeln!(
        buf,
        "/// Single-column expected-value table produced by backward-induction DP.\n\
         /// Index is a 13-bit fill pattern (bit i = 1 → row i is already filled).\n\
         /// `V_COL[0b1_1111_1111_1111]` = 0.0 (all rows filled; no more score possible).\n\
         pub const V_COL: [f32; 8192] = ["
    )
    .unwrap();
    // 8 values per line for readability.
    for chunk in v_col.chunks(8) {
        let line: Vec<String> = chunk.iter().map(|v| format!("{v:.6}_f32")).collect();
        writeln!(buf, "    {},", line.join(", ")).unwrap();
    }
    writeln!(buf, "];\n").unwrap();

    // YZ_BONUS_CORRECTION
    writeln!(
        buf,
        "/// Additive Sixzee-bonus correction for the Ask Grandma advisor.\n\
         ///\n\
         /// Layout: index `6 - n_sixzee_open` for forfeited=false (0..=6),\n\
         ///         index `6 - n_sixzee_open + 7` for forfeited=true (7..=13).\n\
         /// All forfeited entries equal 0.0.\n\
         pub const YZ_BONUS_CORRECTION: [f32; 14] = ["
    )
    .unwrap();
    let yz_strs: Vec<String> = yz.iter().map(|v| format!("{v:.6}_f32")).collect();
    writeln!(buf, "    {},", yz_strs.join(", ")).unwrap();
    writeln!(buf, "];").unwrap();

    std::fs::write(&out_path, buf)?;
    println!("Wrote {}", out_path.display());
    Ok(())
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    println!("rw_sixzee DP precomputation solver");
    println!("===================================");

    let start = std::time::Instant::now();
    let (v_col, yz) = solve();
    println!("Solved in {:.2?}", start.elapsed());

    write_output(&v_col, &yz).expect("failed to write generated/v_col.rs");
    println!("All assertions passed. generated/v_col.rs is ready.");
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_multisets_count() {
        assert_eq!(enumerate_dice_multisets().len(), 252);
    }

    #[test]
    fn dice_probabilities_sum_to_one() {
        let sum: f64 = enumerate_dice_multisets().iter().map(|(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 1e-9, "sum = {sum}");
    }

    #[test]
    fn k_dice_multisets_count() {
        // C(n+k-1, k) for k dice on n=6 faces
        assert_eq!(enumerate_k_dice_multisets(1).len(), 6);
        assert_eq!(enumerate_k_dice_multisets(2).len(), 21);
        assert_eq!(enumerate_k_dice_multisets(3).len(), 56);
        assert_eq!(enumerate_k_dice_multisets(5).len(), 252);
    }

    #[test]
    fn k_dice_probs_sum_to_one() {
        for k in 1..=5 {
            let sum: f64 = enumerate_k_dice_multisets(k).iter().map(|(_, p)| p).sum();
            assert!((sum - 1.0).abs() < 1e-9, "k={k} sum={sum}");
        }
    }

    #[test]
    fn merge_sorted_5_correct() {
        assert_eq!(merge_sorted_5(&[1, 3], &[2, 4, 5]), [1, 2, 3, 4, 5]);
        assert_eq!(merge_sorted_5(&[], &[1, 2, 3, 4, 5]), [1, 2, 3, 4, 5]);
        assert_eq!(merge_sorted_5(&[3, 3, 6], &[1, 4]), [1, 3, 3, 4, 6]);
    }

    #[test]
    fn hold_strategies_exclude_hold_all() {
        let dice = [1u8, 2, 3, 4, 5];
        let holds = enumerate_hold_strategies(dice);
        // Hold-all [1,2,3,4,5] must not appear.
        assert!(!holds.contains(&vec![1, 2, 3, 4, 5]));
        // Hold-nothing [] must appear.
        assert!(holds.contains(&vec![]));
    }

    #[test]
    fn hold_strategies_dedup_repeated_dice() {
        // [3,3,3,5,6]: holding dice[0] or dice[1] or dice[2] all give held=[3].
        let dice = [3u8, 3, 3, 5, 6];
        let holds = enumerate_hold_strategies(dice);
        let count_single_3 = holds.iter().filter(|h| *h == &vec![3]).count();
        assert_eq!(count_single_3, 1, "holding one '3' should be deduplicated to 1 strategy");
    }

    #[test]
    fn score_for_row_ones() {
        assert_eq!(score_for_row(ROW_ONES, [1, 1, 2, 3, 4]), 2);
    }

    #[test]
    fn score_for_row_sixzee() {
        assert_eq!(score_for_row(ROW_SIXZEE, [4, 4, 4, 4, 4]), 50);
        assert_eq!(score_for_row(ROW_SIXZEE, [1, 2, 3, 4, 5]), 0);
    }

    #[test]
    fn yz_bonus_correction_layout() {
        // [0] = n=6 open = 6*15 = 90.0, [6] = n=0 = 0.0, [7..=13] = 0.0
        let (_, yz) = solve();
        assert!((yz[0] - 90.0).abs() < 1e-4);
        assert!((yz[6] - 0.0).abs() < 1e-4);
        for i in 7..14 {
            assert_eq!(yz[i], 0.0, "YZ_BONUS_CORRECTION[{i}] must be 0.0");
        }
    }

    #[test]
    fn v_col_boundary_values() {
        let (v_col, _) = solve();
        assert_eq!(v_col[8191], 0.0, "v_col[8191] must be exactly 0.0");
        assert!(
            (200.0..=300.0).contains(&v_col[0]),
            "v_col[0] = {} out of range",
            v_col[0]
        );
        for (i, &v) in v_col.iter().enumerate() {
            assert!(v.is_finite(), "v_col[{i}] not finite");
        }
    }
}
