# rw_sixzee — Technical Specification

## 1. Overview

rw_sixzee is a client-side-only, solitaire 6-column Sixzee game built with
**Leptos 0.8 (CSR)** compiled to WebAssembly via **Trunk**. It is part of the
Rook Wasteland monorepo and follows the identical project structure, build
toolchain, and conventions used by sibling apps (e.g. rw_teleidoscope).

All game logic, persistence, and Ask Grandma computation run entirely in the
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
│   ├── error.rs            # AppError, AppResult, ErrorSeverity, report_error
│   ├── state/
│   │   ├── mod.rs
│   │   ├── game.rs         # GameState struct + all game logic
│   │   ├── scoring.rs      # scoring functions (pure, no Leptos)
│   │   └── storage.rs      # localStorage read/write (serde_json)
│   │   └── quotes.rs       # QuoteBank loading, tier computation, random selection
│   ├── components/
│   │   ├── mod.rs
│   │   ├── game_view.rs    # full game screen
│   │   ├── dice_row.rs     # 5-die strip with hold toggle
│   │   ├── scorecard.rs    # 6-column scorecard grid
│   │   ├── grandma.rs      # Ask Grandma overlay + Worker bridge
│   │   ├── confirm_zero.rs # zero-score / Sixzee-forfeit prompt
│   │   ├── end_game.rs     # game-complete summary overlay
│   │   ├── resume.rs       # resume-vs-new-game prompt
│   │   ├── history.rs      # history list screen
│   │   ├── history_detail.rs # read-only scorecard snapshot
│   │   ├── settings.rs     # theme picker screen
│   │   ├── error_banner.rs # dismissible degraded-error banner
│   │   └── error_overlay.rs # fatal-error full-screen overlay
│   │   └── grandma_quote.rs # opening overlay + inline quote display
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
│   │   ├── grandma_worker.rs  # Web Worker entry (wasm_bindgen)
│   │   └── messages.rs     # GrandmaRequest / GrandmaResponse types
│   └── router.rs           # hash-based route parsing + Route enum
├── assets/
│   └── grandma_quotes.json # Grandma's quotes by scenario/tier; fetched at runtime
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
| `python make.py test` | `cargo test` (native) + `wasm-pack test --headless --firefox` |

`make.py` follows the same structure used across all Rook Wasteland apps
(see `apps/rw_teleidoscope/make.py` as the canonical reference). It is a
plain Python 3 script with a `ROOT = Path(__file__).parent` anchor, a `_run()`
helper that calls `subprocess.run(..., cwd=ROOT, check=True)`, and one
zero-argument function per target dispatched via `globals().get(target)`.

The `test` target passes `--features wasm-test` to `wasm-pack test` so the
library's `#[wasm_bindgen(start)]` entry point is excluded from the test
binary (prevents duplicate symbol errors — see §17 open questions and
repository lessons).

The `offline/` crate is a standalone workspace member. Its output
(`generated/v_col.rs`) is committed to the repo and included at compile time
via `include!(...)` in the Worker.

`assets/grandma_quotes.json` is served as a static file via Trunk's copy-dir
mechanism (see §14.1). It is available at `/rw_sixzee/assets/grandma_quotes.json`
in production and at `http://localhost:8080/assets/grandma_quotes.json` in
development.

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

The tab bar is rendered in `App` and hidden (CSS `display: none`) during the
following overlays: Resume prompt, Ask Grandma panel, Zero-Score confirmation.
The End-of-Game summary overlay does **not** hide the tab bar — it renders
above the completed scorecard with the tab bar remaining accessible beneath.

---

## 5. Game State

### 5.1 Core Struct

```rust
/// Full serialisable game state. Stored as a single JSON blob in localStorage.
#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
    pub id: String,                      // UUID v4, generated in new_game()
    pub cells: [[Option<u8>; 13]; 6],   // [col][row]; None = empty, Some(v) = filled
    pub dice: [Option<u8>; 5],           // None = unrolled this turn
    pub held: [bool; 5],
    pub rolls_used: u8,                  // 0–3
    pub turn: u32,                       // 1-indexed, increments after each cell placement
    pub bonus_turn: bool,                // set true on bonus Sixzee detection; cleared by start_turn()
    pub bonus_pool: u32,
    pub bonus_forfeited: bool,
    pub started_at: String,              // ISO 8601 timestamp
}
```

Row index mapping (0-based within each column):
```
0  Ones           6  Three of a Kind
1  Twos           7  Four of a Kind
2  Threes         8  Full House
3  Fours          9  Small Straight
4  Fives         10  Large Straight
5  Sixes         11  Sixzee
                 12  Chance
```
The upper-section bonus is **not** stored in `cells` — it is a computed value
(`upper_bonus()`) derived from rows 0–5. All 13 indices (0–12) are scoreable
cells; `cells` contains exactly 13 `Option<u8>` entries per column.

### 5.2 Leptos Signal

`GameState` is held in a single coarse-grained `RwSignal<GameState>` created
at the `App` level and passed to child components via Leptos context
(`provide_context`). Key derived read-only signals computed from this signal
using `Memo<T>`:

- `grand_total: Memo<u32>` — sum of all 6 column totals plus `bonus_pool`.
- `score_preview: Memo<[[u8; 13]; 6]>` — for every cell `(col, row)`, the
  score the current dice would yield. Computed only when `rolls_used > 0`;
  when `rolls_used == 0` all entries are 0 and cells show no preview.

### 5.3 Turn Lifecycle

```
start_turn()
  → rolls_used = 0, dice = [None; 5], held = [false; 5], bonus_turn = false

roll()
  → for each i: if !held[i], dice[i] = rand(1..=6)
  → rolls_used += 1
  → persist()
  → detect_bonus_sixzee()  // may immediately end the turn

detect_bonus_sixzee()
  → if all dice same value
     AND all 6 cells[col][11] are Some(_)  // all Sixzee cells filled (any value, incl. 0)
     → bonus_turn = true
     → if !bonus_forfeited: bonus_pool += 100
     → start_turn()   // resets bonus_turn to false; ends turn with no score phase

place_score(col: usize, row: usize)
  → cells[col][row] = Some(score_preview(col, row))
  → if row == 11 && cells[col][11] == Some(0): bonus_forfeited = true
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
storage calls return `AppError::Storage`. The app-level error signal (§16.7)
receives this as a `Degraded` error and the `ErrorBanner` component (§16.8)
informs the player that state will not be saved. Gameplay continues normally.

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
- `.overlay` / `.overlay--grandma` / `.overlay--end-game` / `.overlay--confirm`
- `.history-list` / `.history-list__row`
- `.settings` / `.settings__theme-grid` / `.settings__theme-card` / `.settings__theme-card--active`
- `.error-banner` / `.error-banner__message` / `.error-banner__dismiss` / `.error-banner__details`
- `.error-overlay` / `.error-overlay__body` / `.error-overlay__action` / `.error-overlay__details`

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

## 10. Ask Grandma

### 10.1 Architecture

```
Main Thread (Leptos)                   Web Worker
──────────────────                     ───────────
GrandmaComponent                       grandma_worker.rs
  │                                       │
  ├─ on mount: spawn_grandma()            │
  │   → Worker::new("./grandma_worker.js")│
  │                                       │
  ├─ on Ask Grandma btn click:            │
  │   → postMessage(GrandmaRequest)  ───→ receive GrandmaRequest
  │                                       │  generate candidate actions
  │                                       │  score each via DP table + MC
  │                                       │  sort top 5
  │   ← postMessage(GrandmaResponse) ←─── postMessage(GrandmaResponse)
  │
  └─ update GrandmaPanel signal
```

The Web Worker is a separate WASM binary (`wasm_bindgen` entry) bundled by
Trunk as a side target. Message passing uses `postMessage` with
`JsValue`-serialised structs (via `serde-wasm-bindgen`).

### 10.2 Message Types

```rust
// worker/messages.rs

#[derive(Serialize, Deserialize)]
pub struct GrandmaRequest {
    pub cells: [[Option<u8>; 13]; 6],
    pub dice: [Option<u8>; 5],
    pub held: [bool; 5],
    pub rolls_used: u8,
    pub bonus_pool: u32,
    pub bonus_forfeited: bool,
}

#[derive(Serialize, Deserialize)]
pub struct GrandmaResponse {
    pub actions: Vec<GrandmaAction>,   // up to 5, sorted by est_final_score desc
}

#[derive(Serialize, Deserialize)]
pub struct GrandmaAction {
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

Ask Grandma uses a precomputed **exact dynamic programming value table** as its
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

For each Ask Grandma request:

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

For reroll actions shown in Grandma's Advice panel (e.g. "Sixzee chance: ~3%"),
probabilities are computed analytically — exact multinomial counts over all
6^k outcomes for the un-held dice — independently of the value table.
Displayed as `~X%` rounded to the nearest integer.

---

## 11. Grandma Quotes

### 11.1 Quote Moments

Grandma speaks at four moments in the game, each drawing from a dedicated pool in
`assets/grandma_quotes.json`:

| Moment | Trigger | Pool key | Display |
|--------|---------|----------|---------|
| Opening | New game starts (before first roll) | `opening` | Full-screen overlay; dismissed to begin |
| Closing | All 78 cells filled | `closing.<tier>` | Inside end-of-game summary overlay |
| Sixzee  | Player rolls all-same dice | `sixzee` | Inline banner near dice row |
| Scratch | Player places a zero | `scratch` | Inside zero-score confirmation prompt |

### 11.2 Performance Tiers

The closing quote tier is computed from the player's final grand total:

```
tier = grand_total / THEORETICAL_MAX_SCORE
```

`THEORETICAL_MAX_SCORE` is a `const u32` in `src/state/quotes.rs`, initially set to
**1200** as a placeholder and calibrated during M3 through playtesting.

| Tier | Score range |
|------|-------------|
| `great` | ≥ 80% |
| `good` | 60–80% |
| `ok` | 40–60% |
| `bad` | 20–40% |
| `really_bad` | < 20% |

### 11.3 JSON Asset Schema

File: `assets/grandma_quotes.json` — fetched once on app load. Cached in-memory as `QuoteBank`.

```json
{
  "version": 1,
  "opening": ["...", "..."],
  "closing": {
    "really_bad": ["...", "..."],
    "bad":        ["...", "..."],
    "ok":         ["...", "..."],
    "good":       ["...", "..."],
    "great":      ["...", "..."]
  },
  "sixzee":  ["...", "..."],
  "scratch": ["...", "..."]
}
```

Minimum pool sizes: `opening` ≥ 15; each `closing.*` tier ≥ 8; `sixzee` and `scratch` ≥ 10.

### 11.4 QuoteBank (`src/state/quotes.rs`)

```rust
#[derive(Deserialize, Clone)]
pub struct QuoteBank {
    pub version:  u32,
    pub opening:  Vec<String>,
    pub closing:  ClosingQuotes,
    pub sixzee:   Vec<String>,
    pub scratch:  Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct ClosingQuotes {
    pub really_bad: Vec<String>,
    pub bad:        Vec<String>,
    pub ok:         Vec<String>,
    pub good:       Vec<String>,
    pub great:      Vec<String>,
}

pub enum PerformanceTier { ReallyBad, Bad, Ok, Good, Great }

pub const THEORETICAL_MAX_SCORE: u32 = 1200; // placeholder; calibrated in M3

pub fn compute_tier(grand_total: u32) -> PerformanceTier { ... }
pub fn pick_quote(pool: &[String]) -> Option<&str>  // random selection; None if pool empty
pub async fn load_quote_bank() -> AppResult<QuoteBank>
```

`load_quote_bank()` fetches `/rw_sixzee/assets/grandma_quotes.json` using `gloo-net`
(or `wasm-bindgen-futures` + `web_sys::fetch`), parses JSON via `serde_json`.
On any failure it returns `AppError::GrandmaQuotes(String)`.

### 11.5 GrandmaQuote Component (`src/components/grandma_quote.rs`)

Two variants:

```rust
/// Full-screen opening quote overlay
#[component]
pub fn GrandmaQuoteOverlay(quote: String, on_dismiss: Callback<()>) -> impl IntoView

/// Small inline quote block (Sixzee / scratch)
#[component]
pub fn GrandmaQuoteInline(quote: String) -> impl IntoView
```

BEM classes: `.grandma-quote-overlay`, `.grandma-quote-inline`, `.grandma-quote__text`,
`.grandma-quote__attribution` ("— Grandma").

### 11.6 Error Handling

`AppError::GrandmaQuotes(String)` has `Degraded` severity. On load failure:
- `RwSignal<Option<QuoteBank>>` stays `None`
- All quote display components render nothing when bank is `None`
- No error banner shown to player (silent omission per PRD Req 39)
- Error logged to console via `web_sys::console::warn_1`

---

## 12. Offline DP Precomputation

### 12.1 Solver (`offline/src/main.rs`)

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

### 12.2 Regenerating the Table

```
cd offline && cargo run --release
```

The generated file changes only if the scoring rules change. It is committed
so the main app build has no dependency on running the offline tool.

---

## 13. Testing Strategy

### 13.1 Unit Tests (native `cargo test`)

Located in `src/state/scoring.rs` (inline `#[cfg(test)]` blocks) and
`src/state/game.rs`. No WASM target needed.

Coverage targets:
- All 13 scoring functions: correct values, edge cases, zero cases
- `upper_bonus` threshold (63 boundary)
- `grand_total` computation
- `detect_bonus_sixzee` logic
- `bonus_forfeited` flag set on Sixzee scratch
- `place_score` cell-fill and turn advancement
- Score persistence (localStorage encode/decode round-trip via
  `serde_json::to_string` / `from_str` without WASM)
- Hash router `parse_hash` for all valid and invalid inputs

### 13.2 WASM Integration Tests (`wasm-pack test`)

Located in `tests/integration.rs`. Must include
`wasm_bindgen_test_configure!(run_in_browser)` at file top.

Coverage targets:
- A complete mini-game (stub 6 turns, verify grand total)
- Zero-score confirmation trigger (cells with score == 0)
- Resume: serialise a `GameState`, reload, verify restoration
- Ask Grandma Worker: postMessage round-trip returns 5 actions with valid structure
- DP table sanity check: `V_col(0b1_1111_1111_1111)` == 0.0; fully-empty column
  value is within the known theoretical range (~254 for optimal single-column play)

---

## 14. Key Dependencies

| Crate | Role |
|-------|------|
| `leptos` 0.8 | UI framework (CSR) |
| `wasm-bindgen` | WASM/JS bridge |
| `web-sys` | DOM APIs (localStorage, Worker, hashchange) |
| `serde` + `serde_json` | GameState serialization |
| `serde-wasm-bindgen` | Worker message serialization |
| `gloo-events` | Event listener wrappers |
| `gloo-net` | HTTP fetch for `grandma_quotes.json` (WASM-compatible) |
| `rand` | Dice RNG (`wasm-bindgen` feature for WASM entropy) |
| `thiserror` | `AppError` derive macro |
| `uuid` | Game IDs for history records |
| **offline only** | |
| *(std only, no extra deps)* | DP solver uses only the Rust standard library |

---

## 15. Deployment

The app is deployed as static files served under the path prefix `/rw_sixzee/`.
No server-side routing rules are required because the hash router handles all
navigation client-side.

### 15.1 Trunk Configuration (`Trunk.toml`)

```toml
[build]
target     = "index.html"
dist       = "dist"
# IMPORTANT: must match the deployment subdirectory exactly so Trunk injects
# correct absolute paths for the WASM binary, JS glue, and CSS.
public_url = "/rw_sixzee/"

[watch]
ignore = ["dist", "doc"]

# Copy the assets/ directory into the dist output so grandma_quotes.json
# (and any future static assets) are served alongside the WASM bundle.
[[copy-dir]]
path = "assets"
```

Setting `public_url` causes Trunk to:
- Inject `<base href="/rw_sixzee/">` into `index.html`
- Rewrite all asset `src`/`href` attributes in the emitted HTML to use absolute
  paths rooted at `/rw_sixzee/`

Without this setting, all asset references are relative (e.g. `./index-abc123.js`)
and the app will fail to load when served from a subdirectory.

### 15.2 Effect on the Web Worker

The Worker is spawned with a relative URL (`./grandma_worker.js`). Because the
browser resolves Worker URLs against the document's base URL (set by
`<base href="/rw_sixzee/">`), this resolves correctly to
`/rw_sixzee/grandma_worker.js` — no hard-coded absolute path is needed in the
Rust code.

### 15.3 No App-Level Base URL Constant

The Rust/WASM code does **not** need to know the base path at runtime:

- The hash router operates on `window.location.hash`, which is path-agnostic.
- `localStorage` keys are prefixed with `rw_sixzee.` (namespace, not path).
- All other resource URLs are either inlined (SVG) or handled by Trunk (assets).

The DP value table (`generated/v_col.rs`) is compiled directly into the Worker
WASM binary as a `const` array — no runtime asset fetch, no extra hosting.

### 15.4 rw_index Registration

`rw_sixzee` must be registered in the sibling launcher app at
`apps/rw_index/apps.json`. This file is a JSON array consumed by `rw_index` to
populate its app grid. Each entry has the following shape (all fields required):

```json
{
  "name":        "Sixzee",
  "slug":        "rw_sixzee",
  "path":        "/rw_sixzee/index.html",
  "icon":        "🎲",
  "tagline":     "Stop trying to force luck on the dice.",
  "description": "Six-column solitaire Sixzee. Seventy-eight cells, each one permanent. The dice don't negotiate. When you're stuck, Ask Grandma — she knows exactly what to do. She always has.",
  "status":      "coming_soon"
}
```

**`status` lifecycle:**
- Set to `"coming_soon"` during development (M1 bootstrap).
- Change to `"live"` when the app is deployed and publicly accessible.

This update is made directly to `apps/rw_index/apps.json` in the monorepo.
No rebuild of `rw_index` is needed for the JSON change to take effect if
`rw_index` fetches `apps.json` at runtime; if it is compiled in, `rw_index`
must be rebuilt and redeployed.

---

## 16. Error Handling

### 16.1 Core Types (`src/error.rs`)

All fallible operations return `AppResult<T>`, a type alias over a single
app-wide error enum. No operation should call `panic!()`, `unwrap()`, or
`expect()` unless it meets the criteria in §16.4.

```rust
pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    /// localStorage access, read, or write failure (includes unavailability).
    #[error("Storage error: {0}")]
    Storage(String),

    /// JSON serialisation or deserialisation failure.
    #[error("JSON error: {0}")]
    Json(String),

    /// Web Worker initialisation or postMessage failure.
    #[error("Ask Grandma worker error: {0}")]
    Worker(String),

    /// Grandma quotes JSON fetch or parse failure.
    #[error("Grandma quotes unavailable: {0}")]
    GrandmaQuotes(String),

    /// A web-sys / DOM API returned an error JsValue.
    #[error("DOM error: {0}")]
    Dom(String),

    /// An internal consistency violation that indicates a programming error.
    #[error("Internal error: {0}")]
    Internal(&'static str),
}
```

`AppError` is `Clone` so it can be stored in a Leptos `RwSignal`. All
string payloads are owned so the type is `'static`.

### 16.2 `From` Implementations

```rust
// serde_json — used in storage.rs for GameState / CompletedGame round-trips
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}

// web-sys JsValue errors — any web-sys call returning Result<_, JsValue>
impl From<web_sys::JsValue> for AppError {
    fn from(v: web_sys::JsValue) -> Self {
        AppError::Dom(
            v.as_string()
                .unwrap_or_else(|| "(non-string JS error)".to_string()),
        )
    }
}
```

These two `From` impls cover the majority of fallible call sites via `?`.
`AppError::Storage` and `AppError::Worker` are constructed manually with
context-specific messages at their call sites.

### 16.3 Error Severity

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Feature degraded but game continues (e.g. storage unavailable, Ask Grandma
    /// worker failed to start). Show a non-blocking banner; do not interrupt play.
    Degraded,

    /// Unexpected failure; game state may be unreliable. Show a blocking overlay
    /// and offer a "Start New Game" escape hatch.
    Fatal,
}

impl AppError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Storage(_)       => ErrorSeverity::Degraded,
            AppError::Worker(_)        => ErrorSeverity::Degraded,
            AppError::GrandmaQuotes(_) => ErrorSeverity::Degraded,
            AppError::Json(_)          => ErrorSeverity::Fatal,
            AppError::Dom(_)           => ErrorSeverity::Fatal,
            AppError::Internal(_)      => ErrorSeverity::Fatal,
        }
    }
}
```

### 16.4 Panic Policy

`unwrap()` is **banned** project-wide. `expect("reason")` is permitted only
at the sites listed below, and must carry a message explaining why the panic
is considered unreachable:

| Site | Justification |
|------|---------------|
| `use_context::<T>().expect("…context must be provided")` | Programming error — context is always set up before any child renders. |
| `include!(…)` for the DP table | Compile-time inclusion; failure is a build error, not a runtime error. |
| Arithmetic on dice values in scoring functions | Dice values are constrained to 1–6 by `roll()` and are never None inside a scoring call. |

All other `expect()` / `unwrap()` calls must be replaced with `?` or explicit
error construction. Enforce this in CI with:

```toml
# Cargo.toml [lints.clippy]
[lints.clippy]
unwrap_used = "deny"
```

`expect_used` is intentionally left at `warn` to allow the permitted sites
above without per-site `#[allow]` noise; code review enforces the policy
for any new `expect()` call.

### 16.5 Functions That Must Return `AppResult`

#### `state/storage.rs`
All public functions return `AppResult`:

```rust
pub fn load_in_progress() -> AppResult<Option<GameState>>
pub fn save_in_progress(state: &GameState) -> AppResult<()>
pub fn clear_in_progress() -> AppResult<()>
pub fn load_history() -> AppResult<Vec<CompletedGame>>
pub fn save_history(history: &[CompletedGame]) -> AppResult<()>
pub fn load_theme() -> AppResult<Option<ThemeId>>
pub fn save_theme(theme: ThemeId) -> AppResult<()>
```

#### `state/game.rs`
State-mutation functions that call `persist()` propagate storage errors:

```rust
pub fn roll(state: &mut GameState) -> AppResult<()>
pub fn place_score(state: &mut GameState, col: usize, row: usize) -> AppResult<()>
```

`start_turn()` and `detect_bonus_sixzee()` are pure in-memory mutations with
no I/O; they return `()`.

#### `worker/mod.rs`

```rust
pub fn spawn_grandma() -> AppResult<Worker>
pub fn post_grandma_request(worker: &Worker, req: &GrandmaRequest) -> AppResult<()>
```

### 16.6 Error Propagation Boundaries

These are the points where `?`-chains terminate and errors are handled rather
than propagated:

| Boundary | Error handling |
|----------|----------------|
| `App` `on_mount` — `load_in_progress()` | Failure → treat as no saved game; post `AppError::Storage` to error signal (Degraded banner). |
| `App` `on_mount` — `load_history()` | Failure → treat as empty history; post Degraded banner. |
| `App` `on_mount` — `load_theme()` | Failure → use default theme; post Degraded banner. |
| `roll()` / `place_score()` — `persist()` failure | Game state in memory remains valid; post Degraded banner ("State won't be saved"). Do **not** abort the roll or score placement. |
| Resume prompt — `load_in_progress()` returns corrupt JSON | Post `AppError::Json` (Fatal); discard the corrupt save; offer "Start New" with an explanatory modal. |
| `spawn_grandma()` failure | Disable the Ask Grandma button; show tooltip "Ask Grandma unavailable". Never blocks gameplay. |
| Any unhandled `AppError::Fatal` in a component | Post to the app-level fatal error signal → `ErrorOverlay` replaces the game screen. |

### 16.7 App-Level Error Signal

A single `RwSignal<Option<AppError>>` is created in `App` and provided via
context. Components write to it via `use_context::<RwSignal<Option<AppError>>>()`.

```rust
// In App component
let app_error: RwSignal<Option<AppError>> = create_rw_signal(None);
provide_context(app_error);
```

Helper used throughout:

```rust
/// Post an error to the app-level signal. Logs to the browser console in all builds.
pub fn report_error(err: AppError) {
    web_sys::console::error_1(&format!("[rw_sixzee] {err:?}").into());
    if let Some(signal) = use_context::<RwSignal<Option<AppError>>>() {
        signal.set(Some(err));
    }
}
```

### 16.8 Visual Error Display

#### `ErrorBanner` (Degraded)

A dismissible banner rendered below the game header, replacing the
`StorageAvailability` ad-hoc signal. Shows when
`app_error.severity() == ErrorSeverity::Degraded`.

```
┌─────────────────────────────────────────────────────────────┐
│  ⚠  Storage unavailable — progress will not be saved.  [ ✕ ]│
└─────────────────────────────────────────────────────────────┘
```

- User-friendly one-line summary drawn from `AppError` `Display` impl.
- `[ ✕ ]` dismisses the banner (clears the signal); it does not reappear
  until the next distinct error.
- In `cfg(debug_assertions)` builds, a `<details>` element below the summary
  exposes the full `{:?}` debug string for use during development.

#### `ErrorOverlay` (Fatal)

Full-screen overlay (same visual level as the end-game overlay) shown when
`app_error.severity() == ErrorSeverity::Fatal`. Blocks all game interaction.

```
╔══════════════════════════════════════════════╗
║  ⛔  Something went wrong                    ║
║                                              ║
║  An unexpected error occurred. Your in-      ║
║  progress game may not be recoverable.       ║
║                                              ║
║  [ Start New Game ]                          ║
║                                              ║
║  ▶ Details  (debug only)                     ║
╚══════════════════════════════════════════════╝
```

- "Start New Game" clears `rw_sixzee.in_progress` from localStorage (best-
  effort, ignoring further errors), resets `GameState`, and clears the error
  signal.
- The `▶ Details` section is only rendered in `cfg(debug_assertions)` builds.
  It shows the full `{:#?}` debug output of the `AppError` value in a
  `<pre>` block, making it easy to copy for bug reports.

#### Ask Grandma Panel Error State

If the Worker is unavailable (spawn failed), the Ask Grandma button renders in a
disabled state with a tooltip: "Ask Grandma unavailable". No overlay is shown —
gameplay is fully unaffected.

If a `post_grandma_request` fails mid-session, Grandma's Advice panel shows an inline
message ("Could not reach Grandma — please try again.") with a retry
button. The error is also posted to `report_error` for console logging but
does not affect the app-level error signal (it is scoped to Ask Grandma).

---

## 17. Open Questions

- **Worker bundling with Trunk**: Trunk's multi-target support for a separate
  Worker WASM binary needs validation. The Worker JS shim (`grandma_worker.js`)
  must be generated and placed in `dist/`. Investigate Trunk `[[bin]]` entries
  or a manual post-build step.
- **Reroll deduplication edge cases**: confirm the held-value-tuple deduplication
  handles all-same-value dice correctly (e.g. [5,5,5,5,5] should collapse to
  just 5 unique hold strategies: hold 0/1/2/3/4 of the 5s).
