# Task M7: Ask Grandma

**Milestone:** M7 — Ask Grandma
**Status:** 🔄 In Progress

## Restatement

Ask Grandma is a purely informational advisory feature that recommends the top 5
moves for the current game state. A Web Worker runs a separate WASM binary that
receives the game state, generates score-now and reroll candidates, ranks them via
the precomputed V_COL DP table, and returns the top 5. The UI shows an overlay
panel with action cards and "Apply this move" buttons. Nothing here affects game
rules; a failure degrades the feature while gameplay continues.

Out of scope: multi-turn lookahead beyond one reroll, exact Worker round-trip
integration tests (covered by E2E).

## Design

### Data flow

1. User rolls dice (`rolls_used` becomes 1–3), Ask Grandma button becomes enabled.
2. User clicks Ask Grandma → `game_view::on_ask_grandma` fires.
3. If worker not yet spawned: `spawn_grandma_worker()` creates `Worker::new('./grandma_worker.js')`,
   wires `onmessage` callback to update `grandma_panel_state` signal.
4. `grandma_panel_state` → `GrandmaPanelState::Loading`, `hide_tab_bar` effect fires.
5. `post_grandma_request(&worker, &req)` posts serialised `GrandmaRequest` to worker.
6. Worker: `worker_start()` receives message, calls `compute_grandma_actions()`, posts
   `GrandmaResponse` back.
7. App `onmessage` handler deserialises response → `grandma_panel_state` → `Ready(actions)`.
8. `GrandmaPanel` re-renders with 5 action cards.
9. User clicks Apply Reroll → sets `game_signal.held`, closes panel.
   User clicks Apply Score-now → calls `place_score` path (zero-confirm if needed), closes panel.
   User clicks ✕ → closes panel.

### Function / type signatures

```rust
// worker/messages.rs
pub struct GrandmaRequest {
    pub cells: [[Option<u8>; 13]; 6],
    pub dice: [u8; 5],
    pub held: [bool; 5],
    pub rolls_used: u8,
    pub bonus_pool: u32,
    pub bonus_forfeited: bool,
}
pub struct GrandmaResponse { pub actions: Vec<GrandmaAction> }
pub struct GrandmaAction {
    pub kind: ActionKind,
    pub description: String,
    pub detail: String,
    pub est_final_score: u32,
}
pub enum ActionKind {
    Reroll { hold_mask: [bool; 5] },
    Score { col: usize, row: usize, points: u8 },
}

// worker/advisor.rs
pub fn compute_grandma_actions(req: &GrandmaRequest) -> GrandmaResponse

// worker/mod.rs
pub enum GrandmaPanelState { Closed, Loading, Ready(Vec<GrandmaAction>), Error(String) }
pub fn spawn_grandma_worker(
    worker_sig: RwSignal<Option<Worker>>,
    panel_state: RwSignal<GrandmaPanelState>,
) -> AppResult<()>
pub fn post_grandma_request(worker: &Worker, req: &GrandmaRequest) -> AppResult<()>

// worker/grandma_worker.rs (feature = "worker")
#[wasm_bindgen(start)]
pub fn worker_start()

// components/grandma.rs
#[component] pub fn GrandmaPanel() -> impl IntoView
```

### Edge cases

- `rolls_used == 0`: Ask Grandma button disabled; no request posted.
- All 78 cells filled (game over): button disabled by `is_game_complete`.
- Worker spawn fails (no Worker API): `grandma_worker = None`, button disabled permanently.
- Worker posts error string: `GrandmaPanelState::Error(msg)` shown with retry button.
- Hold mask with all identical dice (e.g. [5,5,5,5,5]): deduplication yields 5 strategies (hold 0–4).
- Score-now with 0 points: `est_final_score` still computed; Apply triggers zero-confirm.
- `bonus_forfeited = true`: correction index offset by 7.
- Only 1–4 open cells remain: fewer than 5 score-now candidates. Must fill with best rerolls.

### Integration points

- `src/Cargo.toml` — add `worker` feature, web-sys `Worker`/`MessageEvent`/`DedicatedWorkerGlobalScope`
- `src/lib.rs` — gate `main()` with `not(feature = "worker")`
- `src/app.rs` — spawn worker, provide `grandma_worker` + `grandma_panel_state` context
- `src/state/mod.rs` — add `ShowGrandmaPanel` newtype
- `src/worker/mod.rs` — implement bridge; define `GrandmaPanelState`
- `src/components/game_view.rs` — wire Ask Grandma button click
- `make.py` — build worker WASM with `--features worker` + wasm-bindgen-cli

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Race: message posted before worker `onmessage` set | Worker sends "ready" string; request posted lazily on user click (seconds after spawn) |
| Correctness | hold-mask dedup misses identical-dice edge case | Dedup on sorted held-values tuple; tested |
| Simplicity | Two-phase worker protocol (ready + response) | Only "ready" ignored silently; no complex state machine needed |
| Coupling | `GrandmaPanelState` references `GrandmaAction` from worker module | Accepted; both are `wasm32`-gated |
| Performance | MC sampling 300 × k dice × 78 cells in worker thread | Worker thread, non-blocking main; ≪ 50ms |
| Testability | Worker round-trip hard to test in wasm-pack | Computation tested as unit test; E2E covers full round-trip |

## Implementation Notes

- DP formula: `est_final = already_scored + sum_vcol + marginal + bonus_pool + bonus_correction`
  where `bonus_correction = YZ_BONUS_CORRECTION[6 - n_sixzee_open (+ 7 if forfeited)]`
- Reroll expected value: average over all 6^k outcomes of `max_(col,row) (score_for_row + V_COL[new_fill] - V_COL[old_fill])`
- k ≥ 4: use 300 MC samples; k ≤ 3: enumerate exactly
- Probability estimation: always enumerate all 6^k outcomes (≤ 7776) for exact probabilities
- Worker feature-gates: `not(feature = "worker")` guards Leptos start in `lib.rs`;
  `feature = "worker"` enables `worker_start()` start function
- JS loader at `assets/grandma_worker.js` → Trunk copies to `dist/`; calls `importScripts` + `wasm_bindgen`
- `wasm-bindgen --out-name grandma_worker_core` → produces `grandma_worker_core.js` + `grandma_worker_core_bg.wasm`

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `compute_grandma_actions` returns 5 actions | 1 | ✅ | cargo test |
| Score-now candidates cover all open cells | 1 | ✅ | cargo test |
| Reroll dedup: 5 identical dice → 5 strategies | 1 | ✅ | cargo test |
| DP sanity: V_COL[8191] == 0.0 | 1 | ✅ | cargo test |
| DP sanity: V_COL[0] in [200, 300] | 1 | ✅ | cargo test |
| Worker WASM post_message round-trip | 3 | ✅ E2E | Playwright smoke test |
| Panel opens after roll, closes on ✕ | 3 | ✅ E2E | Playwright smoke test |
| Apply Reroll sets held dice | 3 | ✅ E2E | Deferred to M10 detailed E2E |
| Worker spawn failure → button disabled | 1 | ❌ waived | requires mocking Worker constructor |
| Zero-score Score-now → confirm prompt | 3 | ❌ waived | covered by existing confirm_zero tests |

## Test Results

(filled in after Phase 6)

## Review Notes

(filled in after Phase 7)

## Callouts / Gotchas

- `DedicatedWorkerGlobalScope` accessed via `js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>()`
- Worker runs in separate WASM instance; V_COL included via `include!` at compile time
- `wasm-bindgen --target no-modules` output uses IIFE; `wasm_bindgen` global set on `self`
- The worker binary omits Leptos/components (never referenced); linker tree-shakes them
