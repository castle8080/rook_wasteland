//! Message types shared between the main app and the grandma_worker WASM binary.
//!
//! These types are serialised/deserialised via `serde-wasm-bindgen` when
//! crossing the Web Worker boundary.  They are pure-Rust data with no `web_sys`
//! or Leptos dependencies so they compile for all targets.

use serde::{Deserialize, Serialize};

use crate::state::scoring::ROW_COUNT;

// ─── Request ─────────────────────────────────────────────────────────────────

/// Everything the worker needs to compute the top-5 move recommendations.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrandmaRequest {
    /// Current scorecard: `[col 0..5][row 0..12]`.
    pub cells: [[Option<u8>; ROW_COUNT]; 6],
    /// Current dice values (all `Some` after the first roll).
    pub dice: [u8; 5],
    /// Which dice are currently held.
    pub held: [bool; 5],
    /// Number of rolls used this turn (1 or 2 when Ask Grandma is enabled).
    pub rolls_used: u8,
    /// Accumulated bonus pool.
    pub bonus_pool: u32,
    /// Whether the Sixzee bonus pool is permanently forfeited.
    pub bonus_forfeited: bool,
}

// ─── Response ────────────────────────────────────────────────────────────────

/// The worker's reply: up to 5 ranked actions.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrandmaResponse {
    /// Actions sorted by `est_final_score` descending; at most 5 entries.
    pub actions: Vec<GrandmaAction>,
}

/// A single recommended action with its estimated value.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GrandmaAction {
    /// Whether this is a reroll or a score placement.
    pub kind: ActionKind,
    /// Short primary description, e.g. `"Hold [5, 5, 5] — reroll 2 dice"`.
    pub description: String,
    /// Probability info (reroll) or points info (score-now), e.g. `"Sixzee: ~3%  4K: ~22%"`.
    pub detail: String,
    /// Estimated final game score if this action is taken and optimal play follows.
    pub est_final_score: u32,
}

/// Discriminates between a reroll strategy and an immediate score placement.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ActionKind {
    /// Hold the dice indicated by `hold_mask` and reroll the rest.
    Reroll {
        /// `hold_mask[i] = true` → keep die `i`; false → reroll it.
        hold_mask: [bool; 5],
    },
    /// Score the given `(col, row)` cell immediately with the current dice.
    Score {
        /// Column index (0–5).
        col: usize,
        /// Row index (0–12).
        row: usize,
        /// Exact points the current dice would yield in this cell.
        points: u8,
    },
}
