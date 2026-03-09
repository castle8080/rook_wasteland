//! Grandma quote bank — deserialization, tier computation, and random selection.
//!
//! `compute_tier()` and `pick_quote()` are pure and compile on all targets.
//! `load_quote_bank()` is WASM-only (uses gloo_net HTTP).

use crate::error::{AppError, AppResult};
use rand::Rng;
use serde::Deserialize;

// ─── Calibration constants ───────────────────────────────────────────────────

/// Normalisation denominator for tier computation.
///
/// Calibrated from a 5,000-game simulation (M3, greedy strategy):
///
/// | Percentile | Score |
/// |------------|-------|
/// | min        |   683 |
/// | 10th       |   882 |
/// | 25th       |   939 |
/// | median     | 1,005 |
/// | 75th       | 1,075 |
/// | 90th       | 1,146 |
/// | 95th       | 1,190 |
/// | 99th       | 1,297 |
/// | max        | 1,681 |
///
/// A skilled human player scores roughly 5–15 % above the greedy baseline,
/// placing their median near 1,050–1,100 and exceptional games above 1,200.
/// 1,500 sits above the 95th-percentile greedy score so that "great" requires
/// genuine above-average play rather than just average luck.
///
/// Revisit after M5 playtesting and adjust if the tier distribution feels wrong.
pub const THEORETICAL_MAX_SCORE: u32 = 1_500;

// Tier thresholds (integer percentage of THEORETICAL_MAX_SCORE).
// These replace the original 20/40/60/80 placeholders after M3 calibration.
//
// Cutpoints and their expected frequency in greedy-strategy simulations:
//   really_bad  < 48 % →  < 720  →  ~2 %  (bad luck or very poor play)
//   bad        48–57 % →  720–855 →  ~8 %  (below-average game)
//   ok         57–67 % →  855–1005 → ~40 %  (average; simulation median falls here)
//   good       67–80 % → 1005–1200 → ~47 %  (above-average)
//   great       ≥ 80 % →  ≥ 1200  →   ~3 %  greedy / ~15 % skilled play
const TIER_GREAT_PCT: u32 = 80;
const TIER_GOOD_PCT: u32  = 67;
const TIER_OK_PCT: u32    = 57;
const TIER_BAD_PCT: u32   = 48;

// ─── Types ───────────────────────────────────────────────────────────────────

#[derive(Deserialize, Clone, Debug)]
pub struct QuoteBank {
    pub version: u32,
    pub opening: Vec<String>,
    pub closing: ClosingQuotes,
    pub sixzee:  Vec<String>,
    pub scratch: Vec<String>,
    pub quit:    Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ClosingQuotes {
    pub really_bad: Vec<String>,
    pub bad:        Vec<String>,
    pub ok:         Vec<String>,
    pub good:       Vec<String>,
    pub great:      Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformanceTier {
    ReallyBad,
    Bad,
    Ok,
    Good,
    Great,
}

impl PerformanceTier {
    /// Return the matching quote pool from the bank.
    pub fn quotes<'a>(&self, bank: &'a QuoteBank) -> &'a [String] {
        match self {
            PerformanceTier::ReallyBad => &bank.closing.really_bad,
            PerformanceTier::Bad       => &bank.closing.bad,
            PerformanceTier::Ok        => &bank.closing.ok,
            PerformanceTier::Good      => &bank.closing.good,
            PerformanceTier::Great     => &bank.closing.great,
        }
    }
}

// ─── Tier computation ────────────────────────────────────────────────────────

/// Map a final grand total to a performance tier.
///
/// Uses integer percentage (`grand_total * 100 / THEORETICAL_MAX_SCORE`) to
/// avoid floating-point. Saturates at `Great` for scores above the constant.
pub fn compute_tier(grand_total: u32) -> PerformanceTier {
    let pct = grand_total.saturating_mul(100) / THEORETICAL_MAX_SCORE;
    match pct {
        p if p >= TIER_GREAT_PCT => PerformanceTier::Great,
        p if p >= TIER_GOOD_PCT  => PerformanceTier::Good,
        p if p >= TIER_OK_PCT    => PerformanceTier::Ok,
        p if p >= TIER_BAD_PCT   => PerformanceTier::Bad,
        _                        => PerformanceTier::ReallyBad,
    }
}

// ─── Random selection ────────────────────────────────────────────────────────

/// Return a random quote from `pool`, or `None` if the pool is empty.
pub fn pick_quote(pool: &[String]) -> Option<&str> {
    if pool.is_empty() {
        return None;
    }
    let idx = rand::thread_rng().gen_range(0..pool.len());
    Some(&pool[idx])
}

// ─── Async fetch (WASM only) ─────────────────────────────────────────────────

/// Fetch and deserialise `grandma_quotes.json` from the static asset path.
///
/// Returns `AppError::GrandmaQuotes` on any network or parse failure.
/// The caller must treat this error as `Degraded` — the game continues
/// without quotes when the bank is unavailable.
#[cfg(target_arch = "wasm32")]
pub async fn load_quote_bank() -> AppResult<QuoteBank> {
    let url = concat!(
        "/rw_sixzee/assets/grandma_quotes.json?v=",
        env!("GRANDMA_QUOTES_HASH")
    );
    let resp = gloo_net::http::Request::get(url)
        .send()
        .await
        .map_err(|e| AppError::GrandmaQuotes(e.to_string()))?;

    if !resp.ok() {
        return Err(AppError::GrandmaQuotes(format!("HTTP {}", resp.status())));
    }

    resp.json::<QuoteBank>()
        .await
        .map_err(|e| AppError::GrandmaQuotes(e.to_string()))
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Tier boundaries at 48 / 57 / 67 / 80 % of 1500 = 720 / 855 / 1005 / 1200.
    #[test]
    fn tier_boundaries_really_bad() {
        assert_eq!(compute_tier(0),   PerformanceTier::ReallyBad);
        assert_eq!(compute_tier(719), PerformanceTier::ReallyBad);
    }

    #[test]
    fn tier_boundaries_bad() {
        assert_eq!(compute_tier(720), PerformanceTier::Bad);
        assert_eq!(compute_tier(854), PerformanceTier::Bad);
    }

    #[test]
    fn tier_boundaries_ok() {
        assert_eq!(compute_tier(855),  PerformanceTier::Ok);
        assert_eq!(compute_tier(1004), PerformanceTier::Ok);
    }

    #[test]
    fn tier_boundaries_good() {
        assert_eq!(compute_tier(1005), PerformanceTier::Good);
        assert_eq!(compute_tier(1199), PerformanceTier::Good);
    }

    #[test]
    fn tier_boundaries_great() {
        assert_eq!(compute_tier(1200), PerformanceTier::Great);
        assert_eq!(compute_tier(9999), PerformanceTier::Great);
    }

    #[test]
    fn pick_quote_returns_none_for_empty_pool() {
        assert_eq!(pick_quote(&[]), None);
    }

    #[test]
    fn pick_quote_returns_a_pool_member() {
        let pool = vec!["one".to_string(), "two".to_string(), "three".to_string()];
        let q = pick_quote(&pool);
        assert!(q.is_some());
        let q = q.expect("pool is non-empty");
        assert!(pool.iter().any(|s| s.as_str() == q));
    }

    #[test]
    fn pick_quote_from_quit_shaped_pool_returns_member() {
        // Validates that pick_quote works correctly on a quit-pool shaped slice.
        // The live JSON pool (25 entries) is an asset file; pool size is checked
        // by the Python JSON validator in make.py.
        let pool: Vec<String> = vec![
            "You decided you were done. That's a decision.".to_string(),
            "Not every game is worth finishing.".to_string(),
            "Walking away is a choice. You made one.".to_string(),
        ];
        let q = pick_quote(&pool).expect("non-empty pool returns Some");
        assert!(
            pool.iter().any(|s| s.as_str() == q),
            "pick_quote must return a member of the pool"
        );
    }
}
