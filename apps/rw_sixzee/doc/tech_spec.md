# rw_sixzee — Technical Specification

## 1. Overview

rw_sixzee is a client-side-only, solitaire 6-column Sixzee game built with
**Leptos 0.8 (CSR)** compiled to WebAssembly via **Trunk**. It is part of the
Rook Wasteland monorepo and follows the identical project structure, build
toolchain, and conventions used by sibling apps (e.g. rw_teleidoscope).

All game logic, persistence, and advisor computation run entirely in the
browser. No server is required beyond static file hosting.

---

## 2. Repository Layout

```
apps/rw_sixzee/
├── Cargo.toml              # crate root (lib + optional bins)
├── Trunk.toml              # Trunk build config
├── make.py                 # build / test / dist / lint helpers
├── index.html              # Trunk entry point
├── src/
│   ├── lib.rs              # wasm entry, #[wasm_bindgen(start)], cfg gates
│   ├── app.rs              # root App component, hash-router dispatch
│   ├── state/
│   │   ├── mod.rs
│   │   ├── game.rs         # GameState struct + all game logic
│   │   ├── scoring.rs      # scoring functions (pure, no Leptos)
│   │   └── storage.rs      # localStorage read/write (serde_json)
│   ├── components/
│   │   ├── mod.rs
│   │   ├── game_view.rs    # full game screen
│   │   ├── dice_row.rs     # 5-die strip with hold toggle
│   │   ├── scorecard.rs    # 6-column scorecard grid
│   │   ├── advisor.rs      # advisor overlay + Worker bridge
│   │   ├── confirm_zero.rs # zero-score / Sixzee-forfeit prompt
│   │   ├── end_game.rs     # game-complete summary overlay
│   │   ├── resume.rs       # resume-vs-new-game prompt
│   │   ├── history.rs      # history list screen
│   │   ├── history_detail.rs # read-only scorecard snapshot
│   │   └── settings.rs     # theme picker screen
│   ├── dice_svg/
│   │   ├── mod.rs          # DiceFace component dispatch
│   │   ├── devil_rock.rs   # 6 face SVGs for Devil Rock theme
│   │   ├── borg.rs
│   │   ├── horror.rs
│   │   ├── renaissance.rs
│   │   ├── nordic_minimal.rs
│   │   └── pacific_northwest.rs
│   ├── worker/
│   │   ├── mod.rs          # Worker spawn + message bridge
│   │   ├── advisor_worker.rs  # Web Worker entry (wasm_bindgen)
│   │   └── messages.rs     # AdvisorRequest / AdvisorResponse types
│   └── router.rs           # hash-based route parsing + Route enum
├── style/
│   └── main.css            # single flat BEM stylesheet (+ theme vars)
├── tests/
│   └── integration.rs      # WASM integration tests (wasm-pack test)
├── generated/
│   └── v_col.rs            # auto-generated: [f32; 8192] DP value table (committed)
└── offline/                # offline DP precomputation (not shipped)
    ├── Cargo.toml
    └── src/
        └── main.rs         # single-column DP solver → generated/v_col.rs
```

---

## 3. Build Setup

Identical to other monorepo apps:

| Command | Effect |
|---------|--------|
| `python make.py build` | `trunk build` (debug WASM) |
| `python make.py dist` | `trunk build --release` → `dist/` |
| `python make.py lint` | `cargo clippy --target wasm32-unknown-unknown` |
| `python make.py test` | `cargo test` (native) + `wasm-pack test --headless --chrome` |

The `offline/` crate is a standalone workspace member. Its output
(`generated/v_col.rs`) is committed to the repo and included at compile time
via `include!(...)` in the Worker.

---

## 4. Hash-Based Router

### 4.1 Routes

| Hash | Screen |
|------|--------|
| `#/` or `#/game` | Game view (default) |
| `#/history` | History list |
| `#/history/:id` | History detail (scorecard snapshot) |
| `#/settings` | Settings / theme picker |

On app load, `window.location.hash` is parsed once to determine the initial
`Route`. The tab bar updates `window.location.hash` on click; a
`hashchange` listener keeps the active `Route` signal in sync for
browser back/forward navigation.

### 4.2 Route Enum

```rust
pub enum Route {
    Game,
    History,
    HistoryDetail { id: String },
    Settings,
}
```

The `App` component holds a `RwSignal<Route>` and conditionally renders the
matching screen component. No `leptos_router` dependency.

### 4.3 Tab Bar Visibility

The tab bar is rendered in `App` and hidden (CSS `display: none`) during
full-screen overlays (Resume prompt, Advisor panel, End-of-Game summary,
Zero-Score confirmation).

---

## 5. Game State

### 5.1 Core Struct

```rust
/// Full serialisable game state. Stored as a single JSON blob in localStorage.
#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub cells: [[Option<u8>; 13]; 6],   // [col][row]; None = empty, Some(v) = filled
    pub dice: [Option<u8>; 5],           // None = unrolled this turn
    pub held: [bool; 5],
    pub rolls_used: u8,                  // 0–3
    pub turn: u32,                       // 1-indexed, increments after each cell placement
    pub bonus_turn: bool,                // true when current turn is a bonus Sixzee
    pub bonus_pool: u32,
    pub bonus_forfeited: bool,
    pub started_at: String,              // ISO 8601 timestamp
}
```

Row index mapping (0-based within each column):
```
0  Ones          7  Full House
1  Twos          8  Small Straight
2  Threes        9  Large Straight
3  Fours        10  Sixzee
4  Fives        11  Chance
5  Sixes
6  Three of a Kind
```
(Row 12 is reserved for the upper-section bonus display; it is not stored in
`cells` — it is computed on read.)

### 5.2 Leptos Signal

`GameState` is held in a single coarse-grained `RwSignal<GameState>` created
at the `App` level and passed to child components via Leptos context
(`provide_context`). Derived read-only signals (e.g. `grand_total`,
`score_preview`) are computed from this signal using `Memo<T>`.

### 5.3 Turn Lifecycle

```
start_turn()
  → rolls_used = 0, dice = [None; 5], held = [false; 5], bonus_turn = false

roll()
  → for each i: if !held[i], dice[i] = rand(1..=6)
  → rolls_used += 1
  → detect_bonus_sixzee()  // may immediately end the turn

detect_bonus_sixzee()
  → if all dice same value
     AND all 6 cells[col][10] are Some(50)  // all Sixzee cells filled, none scratched
     → if !bonus_forfeited: bonus_pool += 100
     → start_turn()   // ends turn immediately; no score phase

place_score(col: usize, row: usize)
  → cells[col][row] = Some(score_preview(col, row))
  → if row == 10 && cells[col][10] == Some(0): bonus_forfeited = true
  → turn += 1
  → persist()
  → if all 78 cells filled: trigger end_game()
  → else: start_turn()
```

### 5.4 Scoring Functions (`state/scoring.rs`)

All scoring functions are pure (no side effects, no Leptos). They take
`[u8; 5]` dice and return `u8`.

```rust
pub fn score_ones(dice: [u8; 5]) -> u8
pub fn score_twos(dice: [u8; 5]) -> u8
// ... through score_sixes
pub fn score_three_of_a_kind(dice: [u8; 5]) -> u8
pub fn score_four_of_a_kind(dice: [u8; 5]) -> u8
pub fn score_full_house(dice: [u8; 5]) -> u8
pub fn score_small_straight(dice: [u8; 5]) -> u8
pub fn score_large_straight(dice: [u8; 5]) -> u8
pub fn score_sixzee(dice: [u8; 5]) -> u8
pub fn score_chance(dice: [u8; 5]) -> u8

pub fn score_for_row(row: usize, dice: [u8; 5]) -> u8   // dispatcher
pub fn upper_subtotal(col: &[Option<u8>; 13]) -> u16
pub fn upper_bonus(col: &[Option<u8>; 13]) -> u16        // 35 if subtotal ≥ 63, else 0
pub fn lower_subtotal(col: &[Option<u8>; 13]) -> u16
pub fn column_total(col: &[Option<u8>; 13]) -> u16
pub fn grand_total(cells: &[[Option<u8>; 13]; 6], bonus: u32) -> u32
```

These functions are the primary target of native `cargo test` unit tests.

---

## 6. localStorage Schema

All keys are prefixed with `rw_sixzee.`.

| Key | Type | Content |
|-----|------|---------|
| `rw_sixzee.in_progress` | JSON | `GameState` blob, or absent if no game in progress |
| `rw_sixzee.history` | JSON | `Vec<CompletedGame>` sorted by `final_score` desc |
| `rw_sixzee.theme` | String | Theme ID (e.g. `"nordic_minimal"`) |

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct CompletedGame {
    pub id: String,           // UUID v4 (generated at game start)
    pub completed_at: String, // ISO 8601
    pub final_score: u32,
    pub bonus_pool: u32,
    pub bonus_forfeited: bool,
    pub cells: [[Option<u8>; 13]; 6],
}
```

**Pruning**: on every app load and after each game save, history entries with
`completed_at` older than 365 days are removed.

**Unavailability**: if `localStorage` is not available (e.g. private browsing),
all storage calls are no-ops. A `StorageAvailability` signal informs the UI to
show a non-blocking banner.

---

## 7. Hash Router Implementation

```rust
// router.rs
pub fn parse_hash(hash: &str) -> Route {
    match hash.trim_start_matches('#').trim_start_matches('/') {
        "" | "game" => Route::Game,
        "history"   => Route::History,
        s if s.starts_with("history/") => Route::HistoryDetail {
            id: s["history/".len()..].to_owned(),
        },
        "settings"  => Route::Settings,
        _           => Route::Game,
    }
}

pub fn navigate(route: &Route) {
    // updates window.location.hash; triggers hashchange
}
```

The `hashchange` event listener is registered once in `App` on mount and
updates the `RwSignal<Route>`.

---

## 8. CSS Architecture

### 8.1 Single File — BEM Naming

All styles live in `style/main.css`. Class names follow
`block__element--modifier` convention.

Key blocks:
- `.dice-row` / `.dice-row__die` / `.dice-row__die--held`
- `.scorecard` / `.scorecard__cell` / `.scorecard__cell--open` / `.scorecard__cell--preview` / `.scorecard__cell--zero-preview`
- `.scorecard__cell--preview` shows score preview in brackets
- `.tab-bar` / `.tab-bar__item` / `.tab-bar__item--active`
- `.overlay` / `.overlay--advisor` / `.overlay--end-game` / `.overlay--confirm`
- `.history-list` / `.history-list__row`
- `.settings` / `.settings__theme-grid` / `.settings__theme-card` / `.settings__theme-card--active`

### 8.2 Theming with CSS Custom Properties

The `<body>` element carries a `data-theme` attribute (e.g.
`data-theme="devil_rock"`). Each theme is declared as a CSS selector block
that overrides the custom properties defined in `:root`.

```css
:root {
  --color-bg:         #f5f4f0;
  --color-surface:    #ffffff;
  --color-accent:     #4a5568;
  --color-text:       #1a202c;
  --color-held-border:#2d3748;
  --color-preview:    #2b6cb0;
  --font-body:        'Inter', sans-serif;
  --font-display:     'Inter', sans-serif;
}

[data-theme="devil_rock"] {
  --color-bg:         #0a0a0a;
  --color-surface:    #1a0000;
  --color-accent:     #ff2020;
  --color-text:       #f5e642;
  --font-display:     'Metal Mania', cursive;
  /* ... */
}

[data-theme="borg"] { /* ... */ }
[data-theme="horror"] { /* ... */ }
[data-theme="renaissance"] { /* ... */ }
[data-theme="nordic_minimal"] { /* ... */ }
[data-theme="pacific_northwest"] { /* ... */ }
```

Theme switching: on selection, `document.body.dataset.theme` is set in Rust
via `web_sys` and the choice is written to `localStorage`. No page reload.

### 8.3 Responsive Breakpoint

At `max-width: 599px`, the scorecard shifts to the condensed mobile grid:
abbreviated row labels, smaller cells, minimum 44×44 px touch targets enforced
via `min-height` / `padding`.

---

## 9. SVG Dice Faces

### 9.1 Structure

Each theme provides 6 Leptos components — `Face1` through `Face6` — that
return inline SVG (`view!` macro). All share a common 100×100 viewBox.

```rust
// dice_svg/mod.rs
#[component]
pub fn DiceFace(theme: Theme, value: u8) -> impl IntoView {
    match (theme, value) {
        (Theme::DevilRock, 1) => devil_rock::Face1(),
        (Theme::DevilRock, 2) => devil_rock::Face2(),
        // ...
        (Theme::NordicMinimal, v) => nordic_minimal::face(v),
        // ...
    }
}
```

Per-theme modules export individual face components (or a `face(v: u8)`
dispatch function for simpler themes). SVG content is hardcoded — no runtime
asset loading.

### 9.2 Die Held State

The held border is rendered in the parent `DiceRow` component as a CSS class
(`.dice-row__die--held`) that applies a double-border effect, keeping SVG
faces theme-agnostic.

---

## 10. Move Advisor

### 10.1 Architecture

```
Main Thread (Leptos)                   Web Worker
──────────────────                     ───────────
AdvisorComponent                       advisor_worker.rs
  │                                       │
  ├─ on mount: spawn_worker()             │
  │   → Worker::new("./advisor_worker.js")│
  │                                       │
  ├─ on Advisor btn click:                │
  │   → postMessage(AdvisorRequest)  ───→ receive AdvisorRequest
  │                                       │  load ONNX model (once, cached)
  │                                       │  generate candidate actions
  │                                       │  score each via ONNX + rollout
  │                                       │  sort top 5
  │   ← postMessage(AdvisorResponse) ←─── postMessage(AdvisorResponse)
  │
  └─ update AdvisorPanel signal
```

The Web Worker is a separate WASM binary (`wasm_bindgen` entry) bundled by
Trunk as a side target. Message passing uses `postMessage` with
`JsValue`-serialised structs (via `serde-wasm-bindgen`).

### 10.2 Message Types

```rust
// worker/messages.rs

#[derive(Serialize, Deserialize)]
pub struct AdvisorRequest {
    pub cells: [[Option<u8>; 13]; 6],
    pub dice: [Option<u8>; 5],
    pub held: [bool; 5],
    pub rolls_used: u8,
    pub bonus_pool: u32,
    pub bonus_forfeited: bool,
}

#[derive(Serialize, Deserialize)]
pub struct AdvisorResponse {
    pub actions: Vec<AdvisorAction>,   // up to 5, sorted by est_final_score desc
}

#[derive(Serialize, Deserialize)]
pub struct AdvisorAction {
    pub kind: ActionKind,
    pub description: String,           // e.g. "Hold [5, 5, 5] — reroll 2 dice"
    pub detail: String,                // e.g. "Sixzee chance: ~3%  4-of-a-kind: ~22%"
    pub est_final_score: u32,
}

#[derive(Serialize, Deserialize)]
pub enum ActionKind {
    Reroll { hold_mask: [bool; 5] },
    Score  { col: usize, row: usize, points: u8 },
}
```

### 10.3 Value Function — Single-Column DP Table

The advisor uses a precomputed **exact dynamic programming value table** as its
value function. For a single column with `n` cells remaining, `V_col(fill_pattern)`
is the expected total future score from that column assuming optimal play and
fresh random dice each turn.

**State space**: 2¹³ = 8,192 possible fill patterns (one bit per scoreable row).
**Table size**: 8,192 × `f32` = **32 KB** — embedded as a Rust `const` array.

Since all 6 columns share identical rules, a single table covers all of them.

**DP recurrence** (computed offline, `offline/src/main.rs`):

```
// After all 13 cells are filled, no further value remains.
V_col(0b1_1111_1111_1111) = 0.0

// For an unfilled column state, value is the expected score over a fresh turn:
//   Roll 1 → optionally hold → Roll 2 → optionally hold → Roll 3 → must score
V_col(fill) = E_{dice} [ best_turn(fill, dice, rolls_remaining=3) ]

best_turn(fill, dice, r=0) =
    max over open rows of: score(row, dice) + V_col(fill | 1 << row)

best_turn(fill, dice, r>0) =
    max(
        best_turn(fill, dice, 0),                          // score now
        max over hold_masks H of:
            E_{reroll} [ best_turn(fill, dice_after_H, r-1) ]  // reroll
    )
```

The outer expectation over dice is over all 252 distinct dice multisets, each
weighted by its multinomial probability (e.g. five-of-a-kind has probability
6/6^5; a specific mixed roll has probability 5!/n₁!n₂!… / 6^5).

### 10.4 Six-Column Value Decomposition

The 6-column game value is approximated as the **sum of per-column values**
plus a **Sixzee bonus correction**:

```
est_final ≈ already_scored
           + Σ_{col=0..5} V_col(col_fill_pattern)
           + sixzee_bonus_correction(n_yz_open, bonus_pool, forfeited)
```

**Approximation**: this assumes the columns are independent — each will see
fresh random dice for its remaining cells. The only true cross-column coupling
(beyond the Sixzee bonus pool) is opportunity cost: a great roll can only go
to one column. In practice this is a second-order effect because dice are
i.i.d. each turn.

**Sixzee bonus correction**: a small precomputed table indexed by
`(n_sixzee_cells_open: 0–6, forfeited: bool)` giving the expected bonus pool
contribution assuming optimal Sixzee-cell filling. Stored as a `[f32; 14]`
const (7 values × 2 forfeiture states).

### 10.5 Candidate Action Generation and Ranking

For each advisor request:

1. **Score-now candidates**: iterate all 78 cells; collect open ones.
   For each `(col, row)`:
   ```
   marginal = score(row, dice) + V_col(fill | 1<<row) - V_col(fill)
   est_final = already_scored + Σ V_col(col_fills) + marginal_delta + bonus_correction
   ```
   → 1 table lookup per candidate.

2. **Reroll candidates** (only if rolls remain): generate all 2⁵ = 32 hold
   masks; deduplicate by the *sorted tuple of held die values* (e.g. holding
   die-0 and die-2 when both show 5 is identical to holding die-1 and die-4).
   This typically collapses to 10–15 unique strategies.

   For each unique hold strategy with `k` dice unheld:
   - If `6^k ≤ 252`: enumerate all outcomes exactly.
   - If `6^k > 252` (k ≥ 4): sample 300 random outcomes.

   For each dice outcome, compute `max over all open (col,row) of marginal`,
   then average across outcomes.

3. **Sort all candidates** by `est_final` descending; return top 5.

### 10.6 Probability Estimates (Reroll Display)

For reroll actions shown in the advisor panel (e.g. "Sixzee chance: ~3%"),
probabilities are computed analytically — exact multinomial counts over all
6^k outcomes for the un-held dice — independently of the value table.
Displayed as `~X%` rounded to the nearest integer.

---

## 11. Offline DP Precomputation

### 11.1 Solver (`offline/src/main.rs`)

Pure Rust binary. No ML, no simulation — pure backward-induction DP.

**Algorithm**:
1. Iterate fill patterns from fully-filled (all 13 bits set) down to empty.
2. For each pattern, compute `V_col(fill)` via the recurrence in §10.3.
3. The inner expectation over all 252 dice multisets (each weighted by its
   exact probability) is computed once and cached.
4. Hold-mask optimisation uses the same 252-multiset enumeration.

**Runtime**: expected < 1 second on any modern CPU.

**Output**: `generated/v_col.rs` — a Rust source file containing:
```rust
pub const V_COL: [f32; 8192] = [ /* 8192 values */ ];
pub const YZ_BONUS_CORRECTION: [f32; 14] = [ /* 14 values */ ];
```

This file is committed to the repository. It is included in the Worker crate
via `include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/v_col.rs"))`.

### 11.2 Regenerating the Table

```
cd offline && cargo run --release
```

The generated file changes only if the scoring rules change. It is committed
so the main app build has no dependency on running the offline tool.

---

## 12. Testing Strategy

### 12.1 Unit Tests (native `cargo test`)

Located in `src/state/scoring.rs` (inline `#[cfg(test)]` blocks) and
`src/state/game.rs`. No WASM target needed.

Coverage targets:
- All 12 scoring functions: correct values, edge cases, zero cases
- `upper_bonus` threshold (63 boundary)
- `grand_total` computation
- `detect_bonus_sixzee` logic
- `bonus_forfeited` flag set on Sixzee scratch
- `place_score` cell-fill and turn advancement
- Score persistence (localStorage encode/decode round-trip via
  `serde_json::to_string` / `from_str` without WASM)
- Hash router `parse_hash` for all valid and invalid inputs

### 12.2 WASM Integration Tests (`wasm-pack test`)

Located in `tests/integration.rs`. Must include
`wasm_bindgen_test_configure!(run_in_browser)` at file top.

Coverage targets:
- A complete mini-game (stub 6 turns, verify grand total)
- Zero-score confirmation trigger (cells with score == 0)
- Resume: serialise a `GameState`, reload, verify restoration
- Advisor Worker: postMessage round-trip returns 5 actions with valid structure
- DP table sanity check: `V_col(0b1_1111_1111_1111)` == 0.0; fully-empty column
  value is within the known theoretical range (~254 for optimal single-column play)

---

## 13. Key Dependencies

| Crate | Role |
|-------|------|
| `leptos` 0.8 | UI framework (CSR) |
| `wasm-bindgen` | WASM/JS bridge |
| `web-sys` | DOM APIs (localStorage, Worker, hashchange) |
| `serde` + `serde_json` | GameState serialization |
| `serde-wasm-bindgen` | Worker message serialization |
| `gloo-events` | Event listener wrappers |
| `rand` | Dice RNG (`wasm-bindgen` feature for WASM entropy) |
| `uuid` | Game IDs for history records |
| **offline only** | |
| *(std only, no extra deps)* | DP solver uses only the Rust standard library |

---

## 14. Deployment

The app is deployed as static files. Trunk's `dist/` output is served under
`/rw_sixzee/` (or the configured base path). No server-side routing rules are
required because the hash router handles all navigation client-side.

The DP value table (`generated/v_col.rs`) is compiled directly into the Worker
WASM binary as a `const` array — no runtime asset fetch, no extra hosting.

---

## 15. Open Questions

- **Worker bundling with Trunk**: Trunk's multi-target support for a separate
  Worker WASM binary needs validation. The Worker JS shim (`advisor_worker.js`)
  must be generated and placed in `dist/`. Investigate Trunk `[[bin]]` entries
  or a manual post-build step.
- **Reroll deduplication edge cases**: confirm the held-value-tuple deduplication
  handles all-same-value dice correctly (e.g. [5,5,5,5,5] should collapse to
  just 5 unique hold strategies: hold 0/1/2/3/4 of the 5s).
