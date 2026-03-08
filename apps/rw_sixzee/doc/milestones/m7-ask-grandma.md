# M7 — Ask Grandma

<!-- MILESTONE: M7 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** [M4 — DP Precomputation](m4-dp-precomputation.md), [M5 — Core Game UI](m5-core-game-ui.md)
**Required by:** [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Implement Ask Grandma: a Web Worker running a separate WASM binary that receives the current game state, generates
up to 32 hold-mask and score-now candidate actions, ranks them by estimated end-game score using the precomputed DP
table, and returns the top 5 to the main thread for display.

Ask Grandma does not affect game rules — it is purely informational. A failure here degrades one feature; gameplay
continues unaffected.

---

## Success Criteria

- [ ] After the first roll of a turn, clicking the Ask Grandma button opens Grandma's Advice panel (previously a placeholder)
- [ ] The panel shows 5 actions ranked by estimated end-game score, highest first
- [ ] Each "Reroll" action shows: which dice to hold, approximate probability of target combos (~X%), estimated final score
- [ ] Each "Score now" action shows: row name, column number, exact points, estimated final score
- [ ] Clicking "Apply this move" on a Reroll action sets the held dice mask and closes the panel
  (player must then click Roll to execute)
- [ ] Clicking "Apply this move" on a Score-now action places the score (triggering zero-score confirmation
  if applicable) and closes the panel
- [ ] Grandma's Advice panel closes without action via the ✕ button
- [ ] If worker fails to spawn, Ask Grandma button is permanently disabled with tooltip "Ask Grandma unavailable";
  rest of game is unaffected
- [ ] If a worker request fails mid-session, panel shows inline "Could not reach Grandma — please try again"
  with retry button; no fatal error is triggered
- [ ] WASM integration test: postMessage round-trip returns `GrandmaResponse` with exactly 5 `GrandmaAction` entries
- [ ] DP table sanity: `V_COL[0b1_1111_1111_1111] == 0.0`; `V_COL[0]` is in expected theoretical range

---

## Tasks

### Message Types (`src/worker/messages.rs`)

- [ ] Define `GrandmaRequest` struct: `cells`, `dice`, `held`, `rolls_used`, `bonus_pool`, `bonus_forfeited`
- [ ] Define `GrandmaResponse` struct: `actions: Vec<GrandmaAction>` (up to 5)
- [ ] Define `GrandmaAction` struct: `kind: ActionKind`, `description: String`, `detail: String`, `est_final_score: u32`
- [ ] Define `ActionKind` enum: `Reroll { hold_mask: [bool; 5] }`, `Score { col: usize, row: usize, points: u8 }`
- [ ] Derive `Serialize`/`Deserialize` on all types for `serde-wasm-bindgen`

### Worker Entry (`src/worker/grandma_worker.rs`)

- [ ] Add `#[wasm_bindgen(start)]` function that sets a `message` event listener on `self` (the Worker global)
- [ ] On message: deserialise `GrandmaRequest` from `JsValue` (via `serde-wasm-bindgen`);
  run Ask Grandma computation; serialise and `post_message` with `GrandmaResponse`
- [ ] Gate with `#[cfg(target_arch = "wasm32")]`
- [ ] Load `V_COL` and `YZ_BONUS_CORRECTION` via `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/v_col.rs"))`
  (permitted `expect()` / compile-time include site)

### Worker Infrastructure (`src/worker/mod.rs`)

- [ ] Implement `spawn_grandma() -> AppResult<Worker>` — constructs `Worker::new("./grandma_worker.js")`;
  maps `JsValue` error to `AppError::Worker`
- [ ] Implement `post_grandma_request(worker: &Worker, req: &GrandmaRequest) -> AppResult<()>`

### Trunk Multi-Target Setup

- [ ] Configure `Trunk.toml` (or a separate `Trunk-worker.toml`) to build `grandma_worker.rs` as a
  separate WASM binary and place the JS shim (`grandma_worker.js`) in `dist/`
- [ ] Verify `python make.py build` produces both the main app WASM and `dist/grandma_worker.js`

### Candidate Generation & Scoring

- [ ] **Score-now candidates:** iterate all 78 `(col, row)` cells; collect open cells; for each compute:
  ```
  marginal = score_for_row(row, dice) + V_COL[col_fill | 1 << row] - V_COL[col_fill]
  est_final = already_scored + Σ V_COL[col_fills] + marginal + bonus_correction
  ```
  where `already_scored` = sum of all `Some` cells across the board;
  `Σ V_COL[col_fills]` = sum of `V_COL` for each column's current fill pattern
- [ ] **Reroll candidates (if rolls_used < 3):** generate all 2⁵ = 32 hold masks; deduplicate by
  sorted tuple of held die values; for each unique strategy with `k` unheld dice:
  - If `6^k ≤ 252`: enumerate all `6^k` outcomes exactly
  - If `6^k > 252` (k ≥ 4): sample 300 random outcomes (use `rand`)
  - For each outcome compute `max over open (col,row) of marginal`; average all outcomes
- [ ] **Sort** all candidates by `est_final_score` descending; take top 5
- [ ] **Probability estimates** for Reroll display: for each reroll action, compute analytically over all
  `6^k` outcomes the probability of each target combo (Sixzee, 4K, etc.) that appears in `detail` string;
  display as `~X%` rounded to nearest integer

### Ask Grandma Panel (`src/components/grandma.rs`)

- [ ] Spawn worker in `App` on_mount (or lazily on first Ask Grandma click); store in `RwSignal<Option<Worker>>`
- [ ] Ask Grandma button enabled after first roll (`rolls_used > 0`); disabled before
- [ ] Opening Ask Grandma: post `GrandmaRequest`; show loading indicator if response takes >200ms
- [ ] On response: populate Grandma's Advice panel with 5 action cards in ranked order
- [ ] Each card: index (#1–#5), description, detail (probabilities or points), estimated final score, Apply button
- [ ] Apply Reroll: update `held` signal to match `hold_mask`; close panel
- [ ] Apply Score-now: call normal score placement path (which triggers zero-confirm if score == 0); close panel
- [ ] ✕ close button dismisses without action
- [ ] Grandma's Advice panel hides tab bar while open
- [ ] Inline error state: "Could not reach Grandma — please try again" + retry button

### CSS

- [ ] Add `.overlay--grandma` styles: full-screen overlay, scrollable list of 5 action cards
- [ ] Add `.grandma-card`, `.grandma-card__description`, `.grandma-card__detail`, `.grandma-card__score`,
  `.grandma-card__apply`
- [ ] Loading indicator (spinner or pulsing text) for >200ms wait

### WASM Integration Test

- [ ] Test `post_message` round-trip: create a known `GrandmaRequest` (mid-game state, dice rolled),
  send to worker, await response, assert `actions.len() == 5`
- [ ] Test DP sanity: assert `V_COL[8191] == 0.0` and `V_COL[0]` is in [200.0, 300.0]
- [ ] **E2E smoke test** (`e2e/smoke.spec.ts`): after rolling, clicking "Ask Grandma" opens the
  advice panel and displays at least one action card (verifies Worker spawning end-to-end)

---

## Notes & Risks

- **Trunk multi-target is the highest-risk item in this milestone.** The tech spec notes this as an
  open question. Investigate `[[bin]]` entries in Trunk.toml vs. a post-build script approach.
  Fallback: manual `wasm-pack build --target no-modules --out-dir dist/` invoked from `make.py`.
  The `grandma_worker.js` shim must land in `dist/` alongside the main WASM output.
- **Worker global scope:** The grandma worker runs in a `DedicatedWorkerGlobalScope`, not `Window`.
  Avoid any `web-sys` API that requires `Window` (e.g., `window.location`, `document`).
  `console::log_1` is safe; most DOM APIs are not.
- **6-column value decomposition:** Ask Grandma approximates multi-column value as the sum of
  per-column `V_COL` values plus bonus correction. This ignores opportunity cost when a good roll
  fits multiple open columns. The approximation is acceptable per the tech spec (second-order effect).
- **Hold-mask deduplication edge case:** Five identical dice (e.g. [5,5,5,5,5]) must collapse to
  exactly 5 unique strategies: hold 0, 1, 2, 3, or 4 dice. Verify this in tests.
