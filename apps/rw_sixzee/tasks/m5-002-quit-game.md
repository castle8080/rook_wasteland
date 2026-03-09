# Feature 001: Quit Game Mid-Progress

**Feature Doc:** features/feature_001_quit_game_mid_progress.md
**Milestone:** M5 follow-up (Core Game UI gap)
**Status:** ✅ Complete

## Restatement

A ⋮ menu icon in the game header lets the player quit a live game at any time. Tapping it opens a small dropdown; tapping "Quit Game" shows a `ConfirmQuit` overlay with a randomly-drawn Grandma quit quote. Confirming discards the in-progress localStorage save and sets a new `GameActive` context signal to `false`, causing `GameView` to render an `IdleScreen` component instead of the active game. The idle screen shows an opening Grandma quote and a "Start New Game" button, which sets `GameActive` back to `true`, resets game state, and triggers the existing opening-quote overlay flow — identical to the current first-launch experience. The first launch is unaffected: the idle screen only appears after an explicit quit. Out of scope: DNF history, Escape-key dismiss, multi-item menus.

## Design

### Data flow

```
[Player taps ⋮]
  → menu_open.set(true)           (local RwSignal<bool> inside GameMenu)
  → dropdown panel renders

[Player taps "Quit Game" in panel]
  → menu_open.set(false)
  → on_quit_requested.run(())     (Callback from GameView → GameMenu prop)
  → show_confirm_quit.set(true)   (local RwSignal<bool> in GameView)
  → ConfirmQuit overlay renders, picks quit quote from QuoteBank.quit

[Player taps "Keep Playing" or backdrop]
  → show_confirm_quit.set(false)
  → game resumes unchanged

[Player taps "Quit" in ConfirmQuit]
  → storage::clear_in_progress()
  → pending_zero.set(None)
  → sixzee_inline_quote.set(None)
  → game_active.set(false)        (RwSignal<bool> via GameActive context)
  → GameView renders IdleScreen

[Player taps "Start New Game" on IdleScreen]
  → game_signal.set(new_game())
  → game_active.set(true)
  → show_opening_quote.set(true)
  → GameView renders game content; App renders GrandmaQuoteOverlay
```

### Function / type signatures

```rust
// state/mod.rs — new newtype
/// Newtype for the `game_active` signal — `true` while an active game
/// is in progress on the Game tab; `false` shows the idle/pre-game screen.
#[derive(Clone, Copy)]
pub struct GameActive(pub RwSignal<bool>);

// components/confirm_quit.rs
/// Quit-game confirmation overlay.
/// Picks a random quit quote from `QuoteBank.quit`. Backdrop click = cancel.
#[component]
pub fn ConfirmQuit(
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView;

// components/game_menu.rs
/// ⋮ menu icon with a dropdown panel containing "Quit Game".
/// `on_quit_requested` fires when the player selects "Quit Game".
#[component]
pub fn GameMenu(
    on_quit_requested: Callback<()>,
) -> impl IntoView;

// components/idle_screen.rs
/// Pre-game idle screen rendered when no active game is in progress.
/// Picks a random opening quote. `on_start_game` fires on "Start New Game".
#[component]
pub fn IdleScreen(
    on_start_game: Callback<()>,
) -> impl IntoView;
```

### Edge cases

| Case | Handling |
|---|---|
| `QuoteBank` not yet loaded when quit overlay shows | `quit_quote` resolves to `None`; overlay renders without a quote — safe |
| Player taps ⋮ during EndGame overlay | EndGame overlays the whole screen; `GameMenu` is in the header beneath it — unreachable in practice |
| Player taps ⋮ during ConfirmZero overlay | ConfirmZero overlays the screen; unreachable in practice |
| Rapid double-tap on "Quit" | `show_confirm_quit` is set to `false` immediately on confirm, so a second tap is a no-op |
| `storage::clear_in_progress()` fails | Error reported via `report_error()` (Degraded banner); `game_active` still transitions to `false` |
| First app load: `game_active = true` | Idle screen never shows; opening-quote flow is identical to today |

### Integration points

| File | Change |
|---|---|
| `src/state/mod.rs` | Add `GameActive` newtype |
| `src/app.rs` | Create `game_active: RwSignal<bool>`, provide as `GameActive(game_active)` |
| `src/components/mod.rs` | Declare `confirm_quit`, `game_menu`, `idle_screen` modules |
| `src/components/game_view.rs` | Import new components; add `show_confirm_quit` signal; add `GameMenu` to header; wrap content in `game_active` conditional; wire callbacks |
| `src/components/confirm_quit.rs` | New file |
| `src/components/game_menu.rs` | New file |
| `src/components/idle_screen.rs` | New file |
| `style/main.css` | Add `.game-menu`, `.game-menu__btn`, `.game-menu__panel`, `.game-menu__item`, `.game-menu__backdrop`, `.overlay--quit`, `.idle-screen` styles |
| `tests/integration.rs` | 6 new browser integration tests |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `game_active = true` on first load is the right default — no first-load change needed | Confirmed; idle screen shows only after explicit quit |
| Correctness | Storage clear failure must not block the UI transition | `game_active.set(false)` always fires even if `clear_in_progress()` errs |
| Simplicity | Feature doc proposed `RwSignal<Option<GameState>>`; a separate `GameActive` bool is simpler and avoids touching every consumer of `game_signal` | Keep `GameActive` bool signal; leave `game_signal` type unchanged |
| Coupling | `GameView` owns both quit flow *and* game content — it grows larger | Acceptable; callbacks are thin and new overlays are separate components |
| Performance | No reactive performance concern — boolean gate is O(1) | No action needed |
| Testability | Quote selection is random so integration tests cannot assert quote text | Tests assert overlay presence/absence, not quote content |

## Implementation Notes

- Decided NOT to implement Escape-key dismissal for `ConfirmQuit` to avoid the `Send+Sync` issue documented in `lessons.md` L10. Backdrop-click dismissal is implemented instead.
- `GameMenu` dropdown uses a local `menu_open` signal + a backdrop `<div>` for outside-click close — consistent with how `GrandmaQuoteOverlay` handles outside-click dismissal.
- `btn--danger` CSS class added for the destructive "Quit" button (red styling).

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| FR 1: menu icon visible during active game | 3 | ✅ `game_menu_button_present_during_active_game` | |
| FR 2: tapping icon shows options panel | 3 | ✅ `game_menu_panel_shows_on_click` | |
| FR 3: quit option shows ConfirmQuit overlay with quote slot | 3 | ✅ `confirm_quit_overlay_shown_on_quit_tap` | |
| FR 4: cancel returns to game unchanged | 3 | ✅ `cancel_quit_returns_to_active_game` | |
| FR 5: confirm clears storage + sets idle | 3 | ✅ `confirm_quit_shows_idle_screen` | Storage clear failure is Degraded — not unit tested |
| FR 6: idle screen shows on game tab | 3 | ✅ `confirm_quit_shows_idle_screen` | Quote text not asserted (random) |
| FR 7: start new game button triggers game flow | 3 | ✅ `start_new_game_from_idle_returns_to_game` | |
| FR 8: tab bar remains visible on idle screen | — | ⚠️ Waived | Tab bar visibility is CSS-driven, already tested in other tests |
| FR 9: menu icon hidden on idle screen | 3 | ✅ `game_menu_hidden_on_idle_screen` | |
| FR 10: backdrop click cancels overlay | — | ⚠️ Waived | JS click propagation is hard to assert in headless; confirmed by visual smoke test |
| FR 11: quit pool in JSON + deserialized | 1 | ✅ `pick_quote_quit_pool` in `quotes.rs` tests | |
| QuoteBank.quit deserialization | 1 | ✅ covered by existing `pick_quote` generic tests + new pool test | |

## Test Results

- `cargo test`: **93 passed** (93 native unit/integration tests)
- `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings`: **clean**
- `wasm-pack test --headless --firefox`: **43 passed** (6 new quit-game tests included)
- `trunk build`: **success**

## Review Notes

Code-review agent found no genuine bugs, logic errors, or security issues. All signal captures are correct (`RwSignal` is `Copy`). No user-controlled content is rendered (XSS not a concern). Storage errors are caught and reported as Degraded throughout.

## Decisions Made

### Decision: `GameActive` bool vs `RwSignal<Option<GameState>>`
**Chosen:** Separate `GameActive(RwSignal<bool>)` newtype alongside the existing `RwSignal<GameState>`.
**Alternatives considered:** Changing `game_signal` type to `RwSignal<Option<GameState>>`, which would require updating every consumer.
**Rationale:** Minimally invasive — keeps all existing `game_signal` consumers unchanged and avoids a large refactor.

### Decision: Escape key not implemented
**Chosen:** Backdrop-click to cancel only.
**Alternatives considered:** `keydown` event listener on `window` with cleanup.
**Rationale:** The `Send+Sync` constraint on Leptos `on_cleanup` prevents using `gloo_events::EventListener` (lessons.md L10). Implementing it with raw `Closure` + `add_event_listener` + manual cleanup adds significant complexity for a minor UX detail.

### Decision: ⋮ menu icon
**Chosen:** U+22EE vertical ellipsis "⋮" for the game menu trigger.
**Alternatives considered:** ☰ hamburger, ⚙️ gear.
**Rationale:** ⚙️ is already used for the Settings tab — would confuse users. ☰ typically implies navigation. ⋮ is the standard "more options" affordance on mobile.

## Lessons / Highlights

*Filled in after implementation*

## Callouts / Gotchas

- When adding `game_active` to `App`, it must be provided BEFORE components read it, which is the existing pattern.
- `ConfirmQuit` must read `quote_bank` untracked (consistent with `ConfirmZero` and `EndGame`).
- The `IdleScreen` `on_start_game` callback must also reset `pending_zero` and `sixzee_inline_quote` even though they should always be `None` when idle — defensive reset.
