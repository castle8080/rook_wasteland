# M5 — Core Game UI

<!-- MILESTONE: M5 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
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

- [ ] Starting a new game shows 5 unrolled dice, all scorecard cells empty, Roll button enabled, Ask Grandma button disabled
- [ ] Clicking Roll randomises all 5 dice, Roll button remains enabled (up to 3 rolls), Ask Grandma button becomes enabled
- [ ] Clicking a die toggles its held state; held dice show a double-border; subsequent rolls leave held dice unchanged
- [ ] After rolling, every open scorecard cell shows the score the current dice would yield (including 0)
- [ ] Clicking an open cell that scores ≥1 immediately records the score and advances to the next turn
  (dice reset to unrolled, Roll button re-enabled, Ask Grandma button disabled)
- [ ] Clicking an open cell that scores 0 shows the zero-score confirmation prompt; Cancel aborts; Confirm Zero records 0
- [ ] Clicking a Sixzee cell that scores 0 shows the zero-score confirmation WITH the forfeiture warning
- [ ] Filled cells are not clickable
- [ ] Roll button is disabled after 3 rolls in a turn (rolls_used == 3); enabled again after scoring
- [ ] Upper subtotal, upper bonus (+35 if ≥63, else blank), lower subtotal, and column total are
  correctly computed and displayed for each column
- [ ] Grand total shown as sum of all 6 column totals plus Sixzee bonus pool
- [ ] Sixzee bonus pool box shows current total and forfeit state; updates in real time
- [ ] A bonus Sixzee (7th+ roll with all 6 Sixzee cells filled) auto-detects: pool increments by 100,
  turn advances immediately without a score phase, dice reset for fresh turn
- [ ] When all 78 cells are filled, the end-of-game summary overlay appears with final score, best column,
  and "New Game" / "View Full Scorecard" buttons
- [ ] "New Game" from the end-of-game overlay resets the entire game state (M6 will add history save)
- [ ] Turn counter displays current turn number; roll pips reflect remaining rolls

---

## Tasks

### Leptos Signals & Context

- [ ] In `App`, create `RwSignal<GameState>` and provide it via context
- [ ] Create `Memo<u32>` for `grand_total` derived from game state
- [ ] Create `Memo<[[u8; 13]; 6]>` for `score_preview` — computed only when `rolls_used > 0` and all
  dice are `Some`; otherwise all-zeros
- [ ] Provide `grand_total` and `score_preview` memos via context (or pass as props)

### Dice Row (`src/components/dice_row.rs`)

- [ ] Render 5 dice; each die shows its current value (or `?` when `None` / unrolled)
- [ ] Click/tap die toggles `held[i]`; applies `.dice-row__die--held` CSS class for double-border
- [ ] Die faces display numeric value only in M5 (SVG art deferred to M8)
- [ ] Die area has minimum 44×44 px touch target

### Roll & Ask Grandma Buttons

- [ ] Roll button calls `roll()` from game state; disabled when `rolls_used == 3`
- [ ] Ask Grandma button rendered but disabled with placeholder tooltip until M7 is implemented

### Scorecard (`src/components/scorecard.rs`)

- [ ] Render 6-column × 13-row grid plus header row and separator rows
- [ ] Row labels: full names on desktop; abbreviated labels prepared but applied in M10 (responsive CSS)
- [ ] Each cell: if filled (`Some(v)`), display value and `.scorecard__cell--filled`
- [ ] Each open cell after rolling: display preview value in brackets with `.scorecard__cell--preview`;
  zero-preview cells use `.scorecard__cell--zero-preview` distinct styling
- [ ] Each open cell before rolling: blank, styled `.scorecard__cell--open`
- [ ] Clicking open cell: if score > 0, call `place_score()`; if score == 0, open confirm_zero overlay
- [ ] Clicking filled cell: no-op
- [ ] Per-column footer rows: Upper Sub, Bonus (35 or blank), Lower Sub, Col Total
- [ ] Grand total displayed below scorecard
- [ ] Sixzee bonus pool box: shows pool total; "FORFEITED" label when `bonus_forfeited = true`

### Turn Counter & Roll Pips

- [ ] Display current turn number (1-indexed)
- [ ] Display roll pips indicating rolls used (e.g. ●●○ = 2 used, 1 remaining)

### Zero-Score Confirmation (`src/components/confirm_zero.rs`)

- [ ] Overlay shows row name and column number
- [ ] If row == Sixzee (index 11): adds bold forfeiture warning block
- [ ] "Cancel" button dismisses without recording
- [ ] "Confirm Zero" button calls `place_score()` and dismisses
- [ ] Overlay hides tab bar while displayed

### Grandma Quote Components

- [ ] In `App`, spawn async task to call `load_quote_bank()` from `src/state/quotes.rs`;
      on success set `RwSignal<Option<QuoteBank>>`; on failure call `report_error()`
      (Degraded — banner shown, quotes silently omitted, gameplay unaffected)
- [ ] Provide `RwSignal<Option<QuoteBank>>` via context for consumption by child components
- [ ] Implement opening quote overlay (`GrandmaQuoteOverlay`): shown when new game starts
      and QuoteBank is loaded; random pick from `opening` pool; dismissed by button tap or
      outside-card tap; hides tab bar while displayed
- [ ] Implement Sixzee inline quote (`GrandmaQuoteInline`): shown in dice area when all 5
      dice show the same value (`score_sixzee(dice) == 50`); random pick from `sixzee` pool;
      omitted if QuoteBank is `None`
- [ ] Implement scratch quote in `confirm_zero.rs`: random pick from `scratch` pool; rendered
      only if QuoteBank available; displayed between score text and action buttons
- [ ] Implement closing quote in `end_game.rs`: compute performance tier from grand total
      using `compute_tier(grand_total, THEORETICAL_MAX_SCORE)`; random pick from matching
      closing pool; omitted if QuoteBank is `None`
- [ ] All quote displays silently render nothing when `RwSignal<Option<QuoteBank>>` is `None`

### End-of-Game Overlay (`src/components/end_game.rs`)

- [ ] Appears when `is_game_complete()` returns true
- [ ] Shows: column totals summary, Sixzee bonus pool amount, final grand total
- [ ] Highlights best column (highest column total) with label
- [ ] "New Game" button resets `GameState` (calls `GameState::new()`, updates signal)
- [ ] "View Full Scorecard" button navigates to `#/history/:id` (M9 will implement the full view;
  for now it may navigate and show a placeholder)
- [ ] Tab bar remains visible behind overlay

### Grandma Quote Components

- [ ] In `App`, call `load_quote_bank()` inside `spawn_local` on mount; store result in
  `RwSignal<Option<QuoteBank>>`; provide via context
- [ ] `GrandmaQuoteOverlay` component in `src/components/grandma_quote.rs`: full-screen overlay
  shown at game start; picks from `opening` pool; "Let's play." button dismisses (sets
  `RwSignal<bool>` to hide); omit entirely if `QuoteBank` is `None`
- [ ] Opening overlay is shown once per new game; reset signal on `GameState::new()`
- [ ] `GrandmaQuoteInline` component: renders a `👵 "…"` block; used in both Sixzee inline and scratch prompt
- [ ] Sixzee inline quote: in the Sixzee detection path inside `roll()`, pick from `sixzee` pool and
  set a `RwSignal<Option<String>>`; display `GrandmaQuoteInline` near the dice row; auto-dismiss after
  next roll or score action
- [ ] Scratch prompt quote: pass `Option<String>` from `scratch` pool as prop to `ConfirmZero`;
  render `GrandmaQuoteInline` between score text and action buttons; omit if `None`
- [ ] Closing quote: in `EndGame`, read `QuoteBank` from context; call `compute_tier(grand_total)`;
  pick from matching tier pool; display quote block above action buttons; omit if bank unavailable
- [ ] Add CSS: `.grandma-quote-overlay`, `.grandma-quote-overlay__card`, `.grandma-quote__text`,
  `.grandma-quote__attribution`, `.grandma-quote-inline`
- [ ] All `pick_quote()` calls use `Option<&str>` — no `unwrap()`; empty pool silently skipped

### CSS

- [ ] Add scorecard BEM classes to `style/main.css`:
  `.scorecard`, `.scorecard__cell`, `.scorecard__cell--open`, `.scorecard__cell--filled`,
  `.scorecard__cell--preview`, `.scorecard__cell--zero-preview`
- [ ] Add `.dice-row`, `.dice-row__die`, `.dice-row__die--held`
- [ ] Add `.overlay--end-game`, `.overlay--confirm`
- [ ] Add `.grandma-quote-overlay` — full-screen overlay with centred quote card and dismiss button
- [ ] Add `.grandma-quote-inline` — small styled block for Sixzee and scratch quote display
- [ ] Add `.grandma-quote__text` — large italic display font for the quote text
- [ ] Add `.grandma-quote__attribution` — "— Grandma" attribution line
- [ ] Add `.bonus-pool` and `.grand-total` display blocks
- [ ] Alternate column background tones per spec (columns 1,3,5 vs 2,4,6 subtle contrast)
- [ ] Scorecard column headers numbered 1–6

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
