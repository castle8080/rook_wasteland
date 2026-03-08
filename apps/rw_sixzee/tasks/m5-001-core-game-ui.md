# Task M5-001: Core Game UI

**Milestone:** M5 — Core Game UI
**Status:** ✅ Done

## Restatement

M5 builds the complete playable browser game UI for rw_sixzee. It wires the M2
game state and scoring engine into Leptos 0.8 components: a 5-die row with
hold/unhold, a 6-column × 13-row scorecard with live score previews, a Roll
button, zero-score confirmation, Grandma quote displays (opening overlay, Sixzee
inline, scratch, and closing), and an end-of-game summary overlay. The signals
layer (`RwSignal<GameState>`, `Memo<u32>` grand_total, `Memo<[[u8;13];6]>`
score_preview, `RwSignal<Option<QuoteBank>>`) is established in `App` and
provided via context. Five new components are created in `src/components/`:
`dice_row.rs`, `scorecard.rs`, `confirm_zero.rs`, `end_game.rs`, and
`grandma_quote.rs`. The placeholder `game_view.rs` is replaced with a complete
wired game screen. Ask Grandma remains a disabled placeholder button (M7 scope).
Persistence (M6) and SVG dice themes (M8) are out of scope.

## Design

### Data flow

```
User action → game_signal.update(|s| roll/place_score/new_game(s))
           ↓
game_signal (RwSignal<GameState>)
  → Memo<u32> grand_total (scoring::grand_total)
  → Memo<[[u8;13];6]> score_preview (game::score_preview_all)
  ↓
Components read via use_context:
  DiceRow        → game_signal.dice, .held, .rolls_used
  Scorecard      → game_signal.cells, score_preview, grand_total
  ConfirmZero    → game_signal (for row name), quote_bank (scratch quote)
  EndGame        → game_signal, grand_total, quote_bank (closing quote)
  GrandmaQuoteOverlay → quote_bank (opening pool)
```

### Function / type signatures

```rust
// app.rs additions
fn init_game_context()  // creates and provides all context signals

// dice_row.rs
#[component]
pub fn DiceRow() -> impl IntoView

// scorecard.rs
#[component]
pub fn Scorecard(on_cell_click: Callback<(usize, usize)>) -> impl IntoView

// confirm_zero.rs
#[component]
pub fn ConfirmZero(
    col: usize,
    row: usize,
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
) -> impl IntoView

// end_game.rs
#[component]
pub fn EndGame(on_new_game: Callback<()>) -> impl IntoView

// grandma_quote.rs
#[component]
pub fn GrandmaQuoteOverlay(on_dismiss: Callback<()>) -> impl IntoView

#[component]
pub fn GrandmaQuoteInline(quote: String) -> impl IntoView

// game_view.rs
#[component]
pub fn GameView() -> impl IntoView  // complete wired game screen
```

### Edge cases

- `rolls_used == 0`: score_preview is all zeros; no preview shown in cells
- `rolls_used == 3`: Roll button disabled; only cell click ends turn
- Bonus Sixzee: after `roll()`, state.dice is `[None;5]` (start_turn was called
  inside detect_bonus_sixzee); no confirm_zero; no score phase needed
- Opening quote: if QuoteBank is None when show_opening_quote=true, overlay
  is skipped and game starts directly
- Empty quote pools: `pick_quote(&[])` returns None; silently omit quote display
- Clicking already-filled cell: no-op (scorecard renders no click handler)
- Sixzee forfeit: if bonus_forfeited is already true, confirm_zero still shows
  the warning (the forfeit already happened)

### Integration points

- `src/app.rs` — add signals, provide_context, spawn_local, GrandmaQuoteOverlay
- `src/components/mod.rs` — add 5 new pub mod exports
- `src/components/game_view.rs` — complete rewrite
- `src/components/tab_bar.rs` — add hide_tab_bar context check
- `src/components/dice_row.rs` — NEW
- `src/components/scorecard.rs` — NEW
- `src/components/confirm_zero.rs` — NEW
- `src/components/end_game.rs` — NEW
- `src/components/grandma_quote.rs` — NEW
- `style/main.css` — add M5 BEM classes
- `tests/integration.rs` — NEW

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Bonus Sixzee resets dice inside roll(); sixzee inline quote must not trigger | Check `state.dice` after roll — all None means bonus sixzee fired; skip inline quote |
| Simplicity | hide_tab_bar signal adds indirection | Justified by spec requirement; single RwSignal<bool> is minimal |
| Coupling | score_preview Memo duplicates score_preview_all logic | Memo calls score_preview_all() directly — zero duplication |
| Performance | Memo<[[u8;13];6]> recomputes on every game_signal change | Acceptable: scoring is O(78) pure arithmetic, negligible cost |
| Testability | confirm_zero + end_game hard to test without a full game sequence | Integration tests use signal mutation to set state directly |

## Implementation Notes

- Row labels (full names): ["Ones","Twos","Threes","Fours","Fives","Sixes",
  "3 of a Kind","4 of a Kind","Full House","Sm. Straight","Lg. Straight",
  "SIXZEE","Chance"]
- Turn display: state.turn + 1 (1-indexed)
- Roll pips: filled = 3 - rolls_used, empty = rolls_used; symbol ● / ○
- Column header: "C1"…"C6"
- Separator rows: "Upper Sub", "Bonus (+35≥63)", "Lower Sub", "Col Total"
- Bonus pool: shows pool amount; "FORFEITED" suffix if bonus_forfeited=true
- spawn_local comes from wasm_bindgen_futures::spawn_local
- report_error() uses use_context internally — no arg needed

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| Fresh game: dice show ? | 3 | ✅ | integration.rs |
| Roll button enabled at start | 3 | ✅ | integration.rs |
| Roll button disabled after 3 rolls | 3 | ✅ | integration.rs |
| Die click toggles held | 3 | ✅ | integration.rs |
| Scorecard open cell shows preview after roll | 3 | ✅ | integration.rs |
| Clicking cell with score>0 advances turn | 3 | ✅ | integration.rs |
| Game complete triggers end-game overlay | 3 | ✅ | integration.rs |
| confirm_zero shown for zero-score cell | 3 | ✅ | integration.rs |
| Bonus Sixzee advances turn without score phase | 1 | ✅ | existing game.rs tests |
| score_preview memo returns zeros when rolls_used==0 | 1 | via Memo logic | |
| Opening quote shown when bank loaded | 3 | ❌ deferred | load_quote_bank requires network |
| Closing quote tier selection | 1 | ✅ | existing quotes.rs tests |

## Test Results

- `cargo test`: 75/75 native tests pass
- `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings`: clean
- `trunk build`: succeeds, no warnings
- `wasm-pack test --headless --firefox`: 9/9 browser integration tests pass

## Review Notes

Code-review agent flagged `compute_grand_total(&s.cells, s.bonus_pool)` as
potentially including forfeited bonuses. **Waived**: `bonus_pool` only increments
inside `detect_bonus_sixzee` when `!bonus_forfeited`, so `bonus_pool > 0` and
`bonus_forfeited = true` are mutually exclusive — the calculation is correct.

## Callouts / Gotchas

- `detect_bonus_sixzee` is called inside `roll()`, which may call `start_turn()`,
  resetting dice. After `roll()` returns, check `state.dice[0].is_none()` to
  detect if a bonus Sixzee fired.
- `report_error(err)` uses `use_context` internally; it must be called within a
  reactive context (e.g. inside spawn_local that was spawned during component
  rendering, where the reactive owner is still alive).
- The `Memo<[[u8;13];6]>` type is large (78 bytes). Leptos clones on change;
  acceptable since recompute only on dice/cells change.
