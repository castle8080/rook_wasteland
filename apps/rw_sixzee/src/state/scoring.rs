//! Pure scoring functions for rw_sixzee.
//!
//! All functions are side-effect-free and take `[u8; 5]` dice (values 1–6).
//! They are designed to be called from both the game engine and the DP
//! precomputation tool, so they have no dependency on Leptos or web_sys.

// ─── Row-index constants ────────────────────────────────────────────────────

/// Row 0 — Ones (upper section).
pub const ROW_ONES: usize = 0;
/// Row 1 — Twos (upper section).
pub const ROW_TWOS: usize = 1;
/// Row 2 — Threes (upper section).
pub const ROW_THREES: usize = 2;
/// Row 3 — Fours (upper section).
pub const ROW_FOURS: usize = 3;
/// Row 4 — Fives (upper section).
pub const ROW_FIVES: usize = 4;
/// Row 5 — Sixes (upper section).
pub const ROW_SIXES: usize = 5;
/// Row 6 — Three of a Kind (lower section).
pub const ROW_THREE_OF_A_KIND: usize = 6;
/// Row 7 — Four of a Kind (lower section).
pub const ROW_FOUR_OF_A_KIND: usize = 7;
/// Row 8 — Full House (lower section).
pub const ROW_FULL_HOUSE: usize = 8;
/// Row 9 — Small Straight (lower section).
pub const ROW_SMALL_STRAIGHT: usize = 9;
/// Row 10 — Large Straight (lower section).
pub const ROW_LARGE_STRAIGHT: usize = 10;
/// Row 11 — Sixzee (lower section).
pub const ROW_SIXZEE: usize = 11;
/// Row 12 — Chance (lower section).
pub const ROW_CHANCE: usize = 12;

/// Total number of scoreable rows per column.
pub const ROW_COUNT: usize = 13;

/// Human-readable row labels in row-index order (0 = Ones … 12 = Chance).
pub const ROW_LABELS: [&str; ROW_COUNT] = [
    "Ones",
    "Twos",
    "Threes",
    "Fours",
    "Fives",
    "Sixes",
    "3 of a Kind",
    "4 of a Kind",
    "Full House",
    "Sm. Straight",
    "Lg. Straight",
    "SIXZEE",
    "Chance",
];

/// Upper section bonus threshold (sum of rows 0–5 must reach this to earn the bonus).
const UPPER_BONUS_THRESHOLD: u16 = 63;
/// Value of the upper section bonus.
const UPPER_BONUS_VALUE: u16 = 35;
/// Score awarded for a Full House.
const FULL_HOUSE_SCORE: u8 = 25;
/// Score awarded for a Small Straight.
const SMALL_STRAIGHT_SCORE: u8 = 30;
/// Score awarded for a Large Straight.
const LARGE_STRAIGHT_SCORE: u8 = 40;
/// Score awarded for a Sixzee.
const SIXZEE_SCORE: u8 = 50;

// ─── Upper section ──────────────────────────────────────────────────────────

/// Sum of all dice showing 1.
pub fn score_ones(dice: [u8; 5]) -> u8 {
    dice.iter().filter(|&&d| d == 1).sum()
}

/// Sum of all dice showing 2.
pub fn score_twos(dice: [u8; 5]) -> u8 {
    dice.iter().filter(|&&d| d == 2).sum()
}

/// Sum of all dice showing 3.
pub fn score_threes(dice: [u8; 5]) -> u8 {
    dice.iter().filter(|&&d| d == 3).sum()
}

/// Sum of all dice showing 4.
pub fn score_fours(dice: [u8; 5]) -> u8 {
    dice.iter().filter(|&&d| d == 4).sum()
}

/// Sum of all dice showing 5.
pub fn score_fives(dice: [u8; 5]) -> u8 {
    dice.iter().filter(|&&d| d == 5).sum()
}

/// Sum of all dice showing 6.
pub fn score_sixes(dice: [u8; 5]) -> u8 {
    dice.iter().filter(|&&d| d == 6).sum()
}

// ─── Lower section ──────────────────────────────────────────────────────────

/// Sum of all 5 dice if at least 3 show the same value, else 0.
pub fn score_three_of_a_kind(dice: [u8; 5]) -> u8 {
    if has_n_of_a_kind(&dice, 3) {
        dice.iter().sum()
    } else {
        0
    }
}

/// Sum of all 5 dice if at least 4 show the same value, else 0.
pub fn score_four_of_a_kind(dice: [u8; 5]) -> u8 {
    if has_n_of_a_kind(&dice, 4) {
        dice.iter().sum()
    } else {
        0
    }
}

/// 25 if dice contain exactly one pair and one triple (not 5-of-a-kind), else 0.
pub fn score_full_house(dice: [u8; 5]) -> u8 {
    let counts = value_counts(&dice);
    let has_three = counts.contains(&3);
    let has_two = counts.contains(&2);
    if has_three && has_two {
        FULL_HOUSE_SCORE
    } else {
        0
    }
}

/// 30 if dice contain any 4 consecutive distinct values, else 0.
///
/// Qualifying sets include `{1,2,3,4}`, `{2,3,4,5}`, and `{3,4,5,6}`.
pub fn score_small_straight(dice: [u8; 5]) -> u8 {
    let unique = unique_sorted(&dice);
    // Check whether any window of 4 consecutive integers appears in the unique set.
    for start in 1u8..=3 {
        let run: Vec<u8> = (start..start + 4).collect();
        if run.iter().all(|v| unique.contains(v)) {
            return SMALL_STRAIGHT_SCORE;
        }
    }
    0
}

/// 40 if all 5 dice form a single consecutive sequence (1–5 or 2–6), else 0.
pub fn score_large_straight(dice: [u8; 5]) -> u8 {
    let unique = unique_sorted(&dice);
    if unique.len() == 5 && (unique == [1, 2, 3, 4, 5] || unique == [2, 3, 4, 5, 6]) {
        LARGE_STRAIGHT_SCORE
    } else {
        0
    }
}

/// 50 if all 5 dice show the same value, else 0.
pub fn score_sixzee(dice: [u8; 5]) -> u8 {
    if dice.iter().all(|&d| d == dice[0]) {
        SIXZEE_SCORE
    } else {
        0
    }
}

/// Unconditional sum of all 5 dice.
pub fn score_chance(dice: [u8; 5]) -> u8 {
    dice.iter().sum()
}

// ─── Dispatcher ─────────────────────────────────────────────────────────────

/// Return the score for the given row index using the provided dice.
///
/// # Panics
///
/// Panics (debug only) if `row >= ROW_COUNT`. The UI must never pass an
/// out-of-range row — this is a programming error. Using `expect` here is
/// permitted per tech spec §15.4.
pub fn score_for_row(row: usize, dice: [u8; 5]) -> u8 {
    match row {
        ROW_ONES => score_ones(dice),
        ROW_TWOS => score_twos(dice),
        ROW_THREES => score_threes(dice),
        ROW_FOURS => score_fours(dice),
        ROW_FIVES => score_fives(dice),
        ROW_SIXES => score_sixes(dice),
        ROW_THREE_OF_A_KIND => score_three_of_a_kind(dice),
        ROW_FOUR_OF_A_KIND => score_four_of_a_kind(dice),
        ROW_FULL_HOUSE => score_full_house(dice),
        ROW_SMALL_STRAIGHT => score_small_straight(dice),
        ROW_LARGE_STRAIGHT => score_large_straight(dice),
        ROW_SIXZEE => score_sixzee(dice),
        ROW_CHANCE => score_chance(dice),
        _ => panic!("score_for_row: row {row} is out of range (0–12)"),
    }
}

// ─── Column totals ───────────────────────────────────────────────────────────

/// Sum of rows 0–5 (upper section) for a single column.
pub fn upper_subtotal(col: &[Option<u8>; 13]) -> u16 {
    col[0..=5]
        .iter()
        .filter_map(|c| c.map(u16::from))
        .sum()
}

/// 35 if the upper subtotal is ≥ 63, else 0.
pub fn upper_bonus(col: &[Option<u8>; 13]) -> u16 {
    if upper_subtotal(col) >= UPPER_BONUS_THRESHOLD {
        UPPER_BONUS_VALUE
    } else {
        0
    }
}

/// Sum of rows 6–12 (lower section) for a single column.
pub fn lower_subtotal(col: &[Option<u8>; 13]) -> u16 {
    col[6..=12]
        .iter()
        .filter_map(|c| c.map(u16::from))
        .sum()
}

/// Upper subtotal + upper bonus + lower subtotal for a single column.
pub fn column_total(col: &[Option<u8>; 13]) -> u16 {
    upper_subtotal(col) + upper_bonus(col) + lower_subtotal(col)
}

/// Sum of all 6 column totals plus the bonus pool.
pub fn grand_total(cells: &[[Option<u8>; 13]; 6], bonus: u32) -> u32 {
    let cols: u32 = cells.iter().map(|col| u32::from(column_total(col))).sum();
    cols + bonus
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Returns `true` if at least `n` dice show the same value.
fn has_n_of_a_kind(dice: &[u8; 5], n: u8) -> bool {
    value_counts(dice).iter().any(|&c| c >= n)
}

/// Returns an array of per-value counts for values 1–6 (index 0 = value 1).
fn value_counts(dice: &[u8; 5]) -> [u8; 6] {
    let mut counts = [0u8; 6];
    for &d in dice {
        if (1..=6).contains(&d) {
            counts[(d - 1) as usize] += 1;
        }
    }
    counts
}

/// Returns the distinct dice values in sorted order.
fn unique_sorted(dice: &[u8; 5]) -> Vec<u8> {
    let mut v: Vec<u8> = dice.to_vec();
    v.sort_unstable();
    v.dedup();
    v
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Upper section ──

    #[test]
    fn ones_sums_matching_dice() {
        assert_eq!(score_ones([1, 1, 2, 3, 4]), 2);
        assert_eq!(score_ones([1, 1, 1, 1, 1]), 5);
        assert_eq!(score_ones([2, 3, 4, 5, 6]), 0);
    }

    #[test]
    fn twos_sums_matching_dice() {
        assert_eq!(score_twos([2, 2, 3, 4, 5]), 4);
        assert_eq!(score_twos([2, 2, 2, 2, 2]), 10);
        assert_eq!(score_twos([1, 3, 4, 5, 6]), 0);
    }

    #[test]
    fn threes_sums_matching_dice() {
        assert_eq!(score_threes([3, 3, 3, 1, 2]), 9);
        assert_eq!(score_threes([1, 2, 4, 5, 6]), 0);
        assert_eq!(score_threes([3, 3, 3, 3, 3]), 15);
    }

    #[test]
    fn fours_sums_matching_dice() {
        assert_eq!(score_fours([4, 4, 1, 2, 3]), 8);
        assert_eq!(score_fours([1, 2, 3, 5, 6]), 0);
        assert_eq!(score_fours([4, 4, 4, 4, 4]), 20);
    }

    #[test]
    fn fives_sums_matching_dice() {
        assert_eq!(score_fives([5, 5, 5, 1, 2]), 15);
        assert_eq!(score_fives([1, 2, 3, 4, 6]), 0);
        assert_eq!(score_fives([5, 5, 5, 5, 5]), 25);
    }

    #[test]
    fn sixes_sums_matching_dice() {
        assert_eq!(score_sixes([6, 6, 1, 2, 3]), 12);
        assert_eq!(score_sixes([1, 2, 3, 4, 5]), 0);
        assert_eq!(score_sixes([6, 6, 6, 6, 6]), 30);
    }

    // ── Lower section ──

    #[test]
    fn three_of_a_kind_valid() {
        assert_eq!(score_three_of_a_kind([3, 3, 3, 1, 2]), 12);
    }

    #[test]
    fn three_of_a_kind_four_is_also_valid() {
        assert_eq!(score_three_of_a_kind([4, 4, 4, 4, 2]), 18);
    }

    #[test]
    fn three_of_a_kind_zero() {
        assert_eq!(score_three_of_a_kind([1, 2, 3, 4, 5]), 0);
    }

    #[test]
    fn four_of_a_kind_valid() {
        assert_eq!(score_four_of_a_kind([5, 5, 5, 5, 2]), 22);
    }

    #[test]
    fn four_of_a_kind_zero_on_three() {
        assert_eq!(score_four_of_a_kind([3, 3, 3, 1, 2]), 0);
    }

    #[test]
    fn four_of_a_kind_five_of_a_kind_valid() {
        // 5-of-a-kind satisfies ≥4 requirement
        assert_eq!(score_four_of_a_kind([6, 6, 6, 6, 6]), 30);
    }

    #[test]
    fn full_house_valid() {
        assert_eq!(score_full_house([2, 2, 3, 3, 3]), 25);
        assert_eq!(score_full_house([1, 1, 1, 6, 6]), 25);
    }

    #[test]
    fn full_house_zero_no_pair() {
        assert_eq!(score_full_house([1, 2, 3, 4, 5]), 0);
    }

    #[test]
    fn full_house_zero_five_of_a_kind() {
        // 5-of-a-kind has no separate pair — must return 0
        assert_eq!(score_full_house([4, 4, 4, 4, 4]), 0);
    }

    #[test]
    fn small_straight_all_three_runs() {
        assert_eq!(score_small_straight([1, 2, 3, 4, 6]), 30); // 1-2-3-4
        assert_eq!(score_small_straight([1, 2, 3, 4, 5]), 30); // contains 2-3-4-5 too
        assert_eq!(score_small_straight([3, 4, 5, 6, 1]), 30); // 3-4-5-6
    }

    #[test]
    fn small_straight_zero() {
        assert_eq!(score_small_straight([1, 1, 2, 3, 5]), 0);
    }

    #[test]
    fn large_straight_valid() {
        assert_eq!(score_large_straight([1, 2, 3, 4, 5]), 40);
        assert_eq!(score_large_straight([2, 3, 4, 5, 6]), 40);
    }

    #[test]
    fn large_straight_zero() {
        assert_eq!(score_large_straight([1, 2, 3, 4, 6]), 0); // gap
        assert_eq!(score_large_straight([1, 1, 2, 3, 4]), 0); // duplicate
    }

    #[test]
    fn sixzee_valid() {
        assert_eq!(score_sixzee([3, 3, 3, 3, 3]), 50);
        assert_eq!(score_sixzee([6, 6, 6, 6, 6]), 50);
    }

    #[test]
    fn sixzee_zero_not_all_same() {
        assert_eq!(score_sixzee([1, 1, 1, 1, 2]), 0);
    }

    #[test]
    fn chance_sums_all() {
        assert_eq!(score_chance([1, 2, 3, 4, 5]), 15);
        assert_eq!(score_chance([6, 6, 6, 6, 6]), 30);
        assert_eq!(score_chance([1, 1, 1, 1, 1]), 5);
    }

    // ── Dispatcher ──

    #[test]
    fn score_for_row_dispatches_correctly() {
        let dice = [1, 1, 2, 3, 4];
        assert_eq!(score_for_row(ROW_ONES, dice), score_ones(dice));
        assert_eq!(score_for_row(ROW_CHANCE, dice), score_chance(dice));
    }

    // ── Column totals ──

    #[test]
    fn upper_subtotal_sums_rows_0_to_5() {
        let mut col = [None; 13];
        col[ROW_ONES] = Some(3);
        col[ROW_SIXES] = Some(30);
        assert_eq!(upper_subtotal(&col), 33);
    }

    #[test]
    fn upper_subtotal_ignores_lower_rows() {
        let mut col = [None; 13];
        col[ROW_CHANCE] = Some(25); // lower row — must not count
        assert_eq!(upper_subtotal(&col), 0);
    }

    #[test]
    fn upper_subtotal_skips_none_cells() {
        let col = [None; 13]; // all empty
        assert_eq!(upper_subtotal(&col), 0);
    }

    #[test]
    fn lower_subtotal_sums_rows_6_to_12() {
        let mut col = [None; 13];
        col[ROW_THREE_OF_A_KIND] = Some(15);
        col[ROW_CHANCE] = Some(20);
        assert_eq!(lower_subtotal(&col), 35);
    }

    #[test]
    fn lower_subtotal_ignores_upper_rows() {
        let mut col = [None; 13];
        col[ROW_ONES] = Some(5); // upper row — must not count
        assert_eq!(lower_subtotal(&col), 0);
    }

    #[test]
    fn lower_subtotal_zero_when_empty() {
        let col = [None; 13];
        assert_eq!(lower_subtotal(&col), 0);
    }

    #[test]
    fn column_total_includes_upper_bonus_when_threshold_met() {
        let mut col = [None; 13];
        // Upper: 63 exactly → earns +35 bonus
        col[ROW_SIXES] = Some(30);
        col[ROW_FIVES] = Some(25);
        col[ROW_FOURS] = Some(8); // 30+25+8 = 63
        // Lower: Full House = 25
        col[ROW_FULL_HOUSE] = Some(25);
        assert_eq!(column_total(&col), 63 + 35 + 25);
    }

    #[test]
    fn column_total_no_bonus_below_threshold() {
        let mut col = [None; 13];
        col[ROW_SIXES] = Some(12); // upper = 12, no bonus
        col[ROW_CHANCE] = Some(20); // lower = 20
        assert_eq!(column_total(&col), 12 + 20); // upper=12, no bonus, lower=20
    }

    #[test]
    fn upper_bonus_boundary() {
        let mut col = [None; 13];
        // 62 → no bonus
        col[ROW_SIXES] = Some(30); // 30
        col[ROW_FIVES] = Some(25); // 25
        col[ROW_FOURS] = Some(7);  // 7 → total 62
        assert_eq!(upper_bonus(&col), 0);
        // 63 → bonus
        col[ROW_FOURS] = Some(8);
        assert_eq!(upper_bonus(&col), 35);
        // 64 → bonus
        col[ROW_FOURS] = Some(9);
        assert_eq!(upper_bonus(&col), 35);
    }

    #[test]
    fn grand_total_sums_correctly() {
        let mut cells = [[None; 13]; 6];
        // Give each column a known upper total of 10 (no bonus), lower total of 5.
        for col in &mut cells {
            col[ROW_ONES] = Some(10); // 10 in upper
            col[ROW_CHANCE] = Some(5); // 5 in lower
        }
        // column_total per col = 10 + 0 (no bonus) + 5 = 15, × 6 = 90
        // plus bonus pool 200
        assert_eq!(grand_total(&cells, 200), 290);
    }

    #[test]
    fn grand_total_includes_upper_bonus_contribution() {
        let mut cells = [[None; 13]; 6];
        // Col 0: upper = 63 → earns +35; all others empty.
        cells[0][ROW_SIXES] = Some(30);
        cells[0][ROW_FIVES] = Some(25);
        cells[0][ROW_FOURS] = Some(8); // 63
        // col 0 total = 63 + 35 = 98; cols 1–5 total = 0; bonus = 0
        assert_eq!(grand_total(&cells, 0), 98);
    }

    #[test]
    fn grand_total_zero_empty_board() {
        let cells = [[None; 13]; 6];
        assert_eq!(grand_total(&cells, 0), 0);
    }
}
