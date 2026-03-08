# M5 — Core Game UI

<!-- MILESTONE: M5 -->
<!-- STATUS: COMPLETE -->

**Status:** ✅ COMPLETE
**Depends on:** [M2 — Game State & Scoring Engine](m2-scoring-engine.md)
**Required by:** M6, M8, M9, M10

---

## Overview

Build the complete playable browser game. This milestone wires M2's game state and scoring engine into Leptos
components: dice display with hold/unhold, the 6×13 scorecard with live score previews, roll button, score placement,
Sixzee bonus pool display, turn counter, zero-score confirmation, and the end-of-game summary overlay.

After this milestone the game is fully playable from first roll to game completion — just without persistence, themes, or
Ask Grandma.

---

## Success Criteria

- [x] Starting a new game shows 5 unrolled dice, all scorecard cells empty, Roll button enabled, Ask Grandma button disabled
- [x] Clicking Roll randomises all 5 dice, Roll button remains enabled (up to 3 rolls), Ask Grandma button becomes enabled
- [x] Clicking a die toggles its held state; held dice show a double-border; subsequent rolls leave held dice unchanged
- [x] After rolling, every open scorecard cell shows the score the current dice would yield (including 0)
- [x] Clicking an open cell that scores ≥1 immediately records the score and advances to the next turn
  (dice reset to unrolled, Roll button re-enabled, Ask Grandma button disabled)
- [x] Clicking an open cell that scores 0 shows the zero-score confirmation prompt; Cancel aborts; Confirm Zero records 0
- [x] Clicking a Sixzee cell that scores 0 shows the zero-score confirmation WITH the forfeiture warning
- [x] Filled cells are not clickable
- [x] Roll button is disabled after 3 rolls in a turn (rolls_used == 3); enabled again after scoring
- [x] Upper subtotal, upper bonus (+35 if ≥63, else blank), lower subtotal, and column total are
  correctly computed and displayed for each column
- [x] Grand total shown as sum of all 6 column totals plus Sixzee bonus pool
- [x] Sixzee bonus pool box shows current total and forfeit state; updates in real time
- [x] A bonus Sixzee (7th+ roll with all 6 Sixzee cells filled) auto-detects: pool increments by 100,
  turn advances immediately without a score phase, dice reset for fresh turn
- [x] When all 78 cells are filled, the end-of-game summary overlay appears with final score, best column,
  and "New Game" / "View Full Scorecard" buttons
- [x] "New Game" from the end-of-game overlay resets the entire game state (M6 will add history save)
- [x] Turn counter displays current turn number; roll pips reflect remaining rolls

---

## Tasks

### Leptos Signals & Context

- [x] In `App`, create `RwSignal<GameState>` and provide it via context
- [x] Create `Memo<u32>` for `grand_total` derived from game state
- [x] Create `Memo<[[u8; 13]; 6]>` for `score_preview` — computed only when `rolls_used > 0` and all
  dice are `Some`; otherwise all-zeros
- [x] Provide `grand_total` and `score_preview` memos via context (or pass as props)

### Dice Row (`src/components/dice_row.rs`)

- [x] Render 5 dice; each die shows its current value (or `?` when `None` / unrolled)
- [x] Click/tap die toggles `held[i]`; applies `.dice-row__die--held` CSS class for double-border
- [x] Die faces display numeric value only in M5 (SVG art deferred to M8)
- [x] Die area has minimum 44×44 px touch target

### Roll & Ask Grandma Buttons

- [x] Roll button calls `roll()` from game state; disabled when `rolls_used == 3`
- [x] Ask Grandma button rendered but disabled with placeholder tooltip until M7 is implemented

### Scorecard (`src/components/scorecard.rs`)

- [x] Render 6-column × 13-row grid plus header row and separator rows
- [x] Row labels: full names on desktop; abbreviated labels prepared but applied in M10 (responsive CSS)
- [x] Each cell: if filled (`Some(v)`), display value and `.scorecard__cell--filled`
- [x] Each open cell after rolling: display preview value in brackets with `.scorecard__cell--preview`;
  zero-preview cells use `.scorecard__cell--zero-preview` distinct styling
- [x] Each open cell before rolling: blank, styled `.scorecard__cell--open`
- [x] Clicking open cell: if score > 0, call `place_score()`; if score == 0, open confirm_zero overlay
- [x] Clicking filled cell: no-op
- [x] Per-column footer rows: Upper Sub, Bonus (35 or blank), Lower Sub, Col Total
- [x] Grand total displayed below scorecard
- [x] Sixzee bonus pool box: shows pool total; "FORFEITED" label when `bonus_forfeited = true`

### Turn Counter & Roll Pips

- [x] Display current turn number (1-indexed)
- [x] Display roll pips indicating rolls used (e.g. ●●○ = 2 used, 1 remaining)

### Zero-Score Confirmation (`src/components/confirm_zero.rs`)

- [x] Overlay shows row name and column number
- [x] If row == Sixzee (index 11): adds bold forfeiture warning block
- [x] "Cancel" button dismisses without recording
- [x] "Confirm Zero" button calls `place_score()` and dismisses
- [x] Overlay hides tab bar while displayed

### Grandma Quote Components

- [x] In `App`, spawn async task to call `load_quote_bank()` from `src/state/quotes.rs`;
      on success set `RwSignal<Option<QuoteBank>>`; on failure call `report_error()`
      (Degraded — banner shown, quotes silently omitted, gameplay unaffected)
- [x] Provide `RwSignal<Option<QuoteBank>>` via context for consumption by child components
- [x] Implement opening quote overlay (`GrandmaQuoteOverlay`): shown when new game starts
      and QuoteBank is loaded; random pick from `opening` pool; dismissed by button tap or
      outside-card tap; hides tab bar while displayed
- [x] Implement Sixzee inline quote (`GrandmaQuoteInline`): shown in dice area when all 5
      dice show the same value (`score_sixzee(dice) == 50`); random pick from `sixzee` pool;
      omitted if QuoteBank is `None`
- [x] Implement scratch quote in `confirm_zero.rs`: random pick from `scratch` pool; rendered
      only if QuoteBank available; displayed between score text and action buttons
- [x] Implement closing quote in `end_game.rs`: compute performance tier from grand total
      using `compute_tier(grand_total, THEORETICAL_MAX_SCORE)`; random pick from matching
      closing pool; omitted if QuoteBank is `None`
- [x] All quote displays silently render nothing when `RwSignal<Option<QuoteBank>>` is `None`

### End-of-Game Overlay (`src/components/end_game.rs`)

- [x] Appears when `is_game_complete()` returns true
- [x] Shows: column totals summary, Sixzee bonus pool amount, final grand total
- [x] Highlights best column (highest column total) with label
- [x] "New Game" button resets `GameState` (calls `GameState::new()`, updates signal)
- [x] "View Full Scorecard" button navigates to `#/history/:id` (M9 will implement the full view;
  for now it may navigate and show a placeholder)
- [x] Tab bar remains visible behind overlay

### Grandma Quote Components

- [x] In `App`, call `load_quote_bank()` inside `spawn_local` on mount; store result in
  `RwSignal<Option<QuoteBank>>`; provide via context
- [x] `GrandmaQuoteOverlay` component in `src/components/grandma_quote.rs`: full-screen overlay
  shown at game start; picks from `opening` pool; "Let's play." button dismisses (sets
  `RwSignal<bool>` to hide); omit entirely if `QuoteBank` is `None`
- [x] Opening overlay is shown once per new game; reset signal on `GameState::new()`
- [x] `GrandmaQuoteInline` component: renders a `👵 "…"` block; used in both Sixzee inline and scratch prompt
- [x] Sixzee inline quote: in the Sixzee detection path inside `roll()`, pick from `sixzee` pool and
  set a `RwSignal<Option<String>>`; display `GrandmaQuoteInline` near the dice row; auto-dismiss after
  next roll or score action
- [x] Scratch prompt quote: pass `Option<String>` from `scratch` pool as prop to `ConfirmZero`;
  render `GrandmaQuoteInline` between score text and action buttons; omit if `None`
- [x] Closing quote: in `EndGame`, read `QuoteBank` from context; call `compute_tier(grand_total)`;
  pick from matching tier pool; display quote block above action buttons; omit if bank unavailable
- [x] Add CSS: `.grandma-quote-overlay`, `.grandma-quote-overlay__card`, `.grandma-quote__text`,
  `.grandma-quote__attribution`, `.grandma-quote-inline`
- [x] All `pick_quote()` calls use `Option<&str>` — no `unwrap()`; empty pool silently skipped

### CSS

- [x] Add scorecard BEM classes to `style/main.css`:
  `.scorecard`, `.scorecard__cell`, `.scorecard__cell--open`, `.scorecard__cell--filled`,
  `.scorecard__cell--preview`, `.scorecard__cell--zero-preview`
- [x] Add `.dice-row`, `.dice-row__die`, `.dice-row__die--held`
- [x] Add `.overlay--end-game`, `.overlay--confirm`
- [x] Add `.grandma-quote-overlay` — full-screen overlay with centred quote card and dismiss button
- [x] Add `.grandma-quote-inline` — small styled block for Sixzee and scratch quote display
- [x] Add `.grandma-quote__text` — large italic display font for the quote text
- [x] Add `.grandma-quote__attribution` — "— Grandma" attribution line
- [x] Add `.bonus-pool` and `.grand-total` display blocks
- [x] Alternate column background tones per spec (columns 1,3,5 vs 2,4,6 subtle contrast)
- [x] Scorecard column headers numbered 1–6

---

## Notes & Risks

- **Score preview Memo reactivity:** The `score_preview` Memo must only recompute when `dice` or
  `held` change. Use `GameState` fields carefully in Memo closures to avoid over-triggering.
- **Dice value extraction:** Inside `place_score()` and `score_preview_all()`, dice are unwrapped.
  This is a permitted `expect()` site per §15.4 — dice are guaranteed `Some` when rolls_used > 0.
- **Bonus Sixzee mid-turn detection:** `detect_bonus_sixzee()` is called after every `roll()` invocation,
  including Roll 1 and Roll 2 (not just Roll 3). The turn ends immediately if detected.
- **End-of-game history save:** M5's "New Game" button does NOT save to history — that is M6's scope.
  Clicking "New Game" simply resets state in memory.
- **E2E infrastructure bootstrapped in M5:** `python make.py e2e` runs Playwright smoke tests
  against `trunk serve`. See `e2e/smoke.spec.ts` and `doc/tech_spec.md §13.3`. Future milestones
  should add E2E smoke tests for any new screen or major user flow they introduce.

---

## Implementation Summary

### What was built

Five new Leptos components were created from scratch, plus a complete rewrite of the `game_view.rs`
placeholder. `App` grew from a thin router shell into a full signal provider: it creates
`RwSignal<GameState>`, a `Memo<u32>` for `grand_total`, a `Memo<[[u8;13];6]>` for `score_preview`,
and a `RwSignal<Option<QuoteBank>>` loaded via `spawn_local` on mount. All are distributed via
`provide_context` so leaf components can read them without prop drilling.

| Component | File | Responsibility |
|---|---|---|
| `DiceRow` | `src/components/dice_row.rs` | 5-die row; click-to-hold toggles; `--held`/`--unrolled` CSS states |
| `Scorecard` | `src/components/scorecard.rs` | 6×13 grid; open/preview/zero-preview/filled cell states; footer rows; grand total |
| `ConfirmZero` | `src/components/confirm_zero.rs` | Zero-score confirmation overlay; Sixzee forfeiture warning; scratch quote |
| `EndGame` | `src/components/end_game.rs` | Game-complete summary overlay; tier-based closing quote; New Game reset |
| `GrandmaQuoteOverlay` / `GrandmaQuoteInline` | `src/components/grandma_quote.rs` | Full-screen opening quote; inline scratch/Sixzee quote block |
| `GameView` | `src/components/game_view.rs` | Central game screen; wires roll, hold, score-placement, bonus detection |

`style/main.css` grew by ~350 lines of BEM CSS covering all new components. `src/state/scoring.rs`
gained `ROW_LABELS: [&str; 13]` used by both `Scorecard` and `ConfirmZero`.

### Interesting decisions

**`score_preview` as a context Memo, not a prop.** Passing a `[[u8;13];6]` matrix as a prop to
`Scorecard` would have required threading it through `GameView` unnecessarily. Providing it via
context keeps `GameView` thin and lets `Scorecard` subscribe directly. The Memo only recomputes
when `dice` or `rolls_used` change, keeping reactivity tight.

**Opening quote as an early return in `App`.** When `show_opening_quote && bank_ready`, `App`
returns `GrandmaQuoteOverlay` via `.into_any()` before rendering the game shell. This keeps the
quote logic out of `GameView` entirely, but it means `.game-header` is absent from the DOM until
the quote is dismissed — a gotcha that affected both integration tests and Playwright smoke tests
(see L12 / smoke test 3).

**`bonus_pool > 0` and `bonus_forfeited = true` are mutually exclusive by construction.** A code
review surfaced this as an apparent bug, but analysis confirmed the invariant holds: `bonus_pool`
is only incremented behind `if !bonus_forfeited`, and forfeiture can only be triggered before any
bonus turns are earned. Documented in L11.

### E2E testing layer added

Playwright was bootstrapped as a second test layer alongside the wasm-pack browser tests.
`python make.py e2e` starts `trunk serve --no-autoreload` (or reuses an existing instance),
then runs 6 smoke tests in Chromium:

1. HTTP 200 response
2. Page title is SIXZEE
3. WASM initialises and renders app UI
4. Five dice are visible after dismissing the opening quote
5. No uncaught JS errors on load
6. `grandma_quotes.json` asset loads (opening quote shown)

Three non-obvious gotchas were encountered and documented in `doc/lessons.md L12`:

- **Trunk `canonicalize()` on watch-ignore paths** — any path in `[watch] ignore` must exist on
  disk. Playwright deletes `test-results/` before spawning the webServer, so the directory must
  NOT be in the ignore list.
- **`waitUntil: "networkidle"` required** — the WASM binary is fetched via dynamic import after
  the HTML `load` event; bare `goto()` returns before WASM is ready.
- **`--no-autoreload` required** — Trunk's live-reload WebSocket re-loads pages when Playwright
  writes result files mid-test, causing intermittent failures.

### Test results at completion

| Suite | Result |
|---|---|
| `cargo test` (native) | 75 / 75 ✅ |
| `wasm-pack test --headless --firefox` | 12 / 12 ✅ |
| `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings` | clean ✅ |
| `trunk build` | clean ✅ |
| `python make.py e2e` (Playwright) | 6 / 6 ✅ |
