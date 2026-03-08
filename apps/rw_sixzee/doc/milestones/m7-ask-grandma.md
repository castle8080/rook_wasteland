# M7 — Ask Grandma

<!-- MILESTONE: M7 -->
<!-- STATUS: DONE -->

**Status:** ✅ DONE
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

- [x] Define `GrandmaRequest` struct: `cells`, `dice`, `held`, `rolls_used`, `bonus_pool`, `bonus_forfeited`
- [x] Define `GrandmaResponse` struct: `actions: Vec<GrandmaAction>` (up to 5)
- [x] Define `GrandmaAction` struct: `kind: ActionKind`, `description: String`, `detail: String`, `est_final_score: u32`
- [x] Define `ActionKind` enum: `Reroll { hold_mask: [bool; 5] }`, `Score { col: usize, row: usize, points: u8 }`
- [x] Derive `Serialize`/`Deserialize` on all types for `serde-wasm-bindgen`

### Worker Entry (`src/worker/grandma_worker.rs`)

- [x] Add `#[wasm_bindgen(start)]` function that sets a `message` event listener on `self` (the Worker global)
- [x] On message: deserialise `GrandmaRequest` from `JsValue` (via `serde-wasm-bindgen`);
  run Ask Grandma computation; serialise and `post_message` with `GrandmaResponse`
- [x] Gate with `#[cfg(all(target_arch = "wasm32", feature = "worker"))]`
- [x] Load `V_COL` and `YZ_BONUS_CORRECTION` via module-wrapped `include!` with `#![allow]` suppressing generated-file lints

### Worker Infrastructure (`src/worker/mod.rs`)

- [x] Implement `spawn_grandma_worker()` — constructs `Worker::new("./grandma_worker.js")`;
  wires `onmessage` to update `grandma_panel_state`; maps error to `AppError::Worker`
- [x] Implement `post_grandma_request(worker: &Worker, req: &GrandmaRequest) -> AppResult<()>`

### Build Setup

- [x] `Cargo.toml`: add `worker = []` feature; add web-sys Worker/MessageEvent/DedicatedWorkerGlobalScope features
- [x] `lib.rs`: gate `#[wasm_bindgen(start)]` to exclude `feature = "worker"`
- [x] `make.py`: `_build_worker(release)` using `cargo build --features worker` + `wasm-bindgen --target no-modules`
  called from both `build()` and `dist()`
- [x] `assets/grandma_worker.js`: JS loader (`importScripts` + `wasm_bindgen`)

### Candidate Generation & Scoring (`src/worker/advisor.rs`)

- [x] Score-now candidates: marginal = `score_for_row + V_COL[fill|1<<row] - V_COL[fill]`
- [x] Reroll candidates: 32 hold masks, dedup by sorted held-values key, exact EV for k≤3, MC (300 samples) for k≥4
- [x] Sort by `est_final_score` descending; take top 5
- [x] Probability detail strings over full 6^k enumeration
- [x] Unit tests: DP sanity, returns-5-actions, five-identical-dice dedup, sorted-board

### App Wiring (`src/app.rs`)

- [x] Spawn worker eagerly; provide `grandma_worker` + `grandma_panel_state` context
- [x] Update `hide_tab_bar` Effect to include grandma panel open state

### Ask Grandma Panel (`src/components/grandma.rs` + `game_view.rs`)

- [x] `GrandmaPanel` component: Loading spinner / Ready cards / Error + retry / ✕ close
- [x] 5 action cards: rank, description, detail, estimated score, Apply button
- [x] Apply Reroll: update `held` mask; close panel
- [x] Apply Score-now (nonzero): place score, persist, close panel
- [x] Apply Score-now (zero): close panel; user must confirm by clicking cell normally
- [x] Ask Grandma button wired in `GameView`: enabled after first roll, disabled if worker absent
- [x] `components/mod.rs` updated

### CSS

- [x] `.overlay--grandma` full-screen overlay with scrollable panel
- [x] `.grandma-card` grid layout (rank / description / detail / score / apply)
- [x] `.grandma-spinner` CSS animation
- [x] Mobile responsive tweaks

### Tests

- [x] `cargo test` — 87 unit tests pass (includes advisor DP sanity + 5-action return)
- [x] `cargo clippy --target wasm32-unknown-unknown -- -D warnings` — clean
- [x] `trunk build` — succeeds
- [x] E2E smoke tests added (`e2e/smoke.spec.ts` M7 block): button state, overlay appears, cards present, close works

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

---

## Implementation Notes

- **Worker binary approach:** Used `--features worker` Cargo feature to build a second WASM entry point
  from the same crate. `lib.rs` `main()` is gated `not(feature = "worker")`; `grandma_worker.rs`
  `worker_start()` is gated `feature = "worker"`. `make.py _build_worker()` runs
  `cargo build --features worker` + `wasm-bindgen --target no-modules --out-name grandma_worker_core`.

- **Generated file lints:** `v_col.rs` triggers `clippy::large_const_arrays` and
  `clippy::excessive_precision`. Fixed by wrapping the `include!` in a `dp_tables` submodule with
  `#![allow(clippy::large_const_arrays, clippy::excessive_precision)]`.

- **Score-now with zero confirmation:** When the advisor recommends a Score-now action that would score
  0 points, clicking Apply closes the panel without placing the score. The player must click the cell
  manually to trigger the normal zero-confirm flow. This is intentional: auto-placing a zero without
  confirmation would violate the game-rule invariant enforced by `ConfirmZero`.

- **Worker "ready" ping:** `worker_start()` posts the string `"ready"` after wiring `onmessage`.
  The main-thread handler silently ignores it (string check before `serde_wasm_bindgen::from_value`).

- **`current_dice` visibility:** Made `pub` so `grandma.rs` and future consumers can extract dice
  without duplicating the `Option`-unwrap pattern.

