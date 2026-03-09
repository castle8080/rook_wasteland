# Feature 001 — Quit Game Mid-Progress

## Status
Implemented

## Summary
Players can quit an active game at any time via an expandable menu icon in the game header. Quitting requires confirmation to prevent accidental loss, discards the in-progress save, and lands the player on a new idle/pre-game screen with a Grandma welcome quote and a "Start New Game" button.

## Problem Statement
Once a game is started in rw_sixzee, the only way to begin a fresh game is to play all 78 cells to completion and click "New Game" from the end-game overlay. A player who wants to abandon a bad run, experiment with a different strategy, or simply start over has no escape hatch. PRD Requirement 20 states that a player shall be able to start a new game at any time — this feature fulfills that requirement with a safe, intentional flow.

## Goals
- A player can quit any active game from the game screen without having to finish it.
- The quit action is protected by a confirmation step so accidental taps don't lose progress silently.
- After quitting, the player lands on an idle screen (Game tab) with a clear path to starting a new game.
- The in-progress localStorage save is fully discarded on confirm — no ghost state persists.
- The feature introduces no regressions to the existing new-game, resume, or end-game flows.

## Non-Goals
- Saving abandoned games to history with a DNF marker — discarded games are gone.
- Pausing a game and resuming it later via a different mechanism (the existing localStorage resume already covers this).
- Adding multiple menu items beyond "Quit Game" in the initial release.
- Modifying the Settings tab or the History tab as part of this feature.

## User Stories
- As a player mid-game, I want to quit my current game so that I can start fresh without having to finish a losing run.
- As a player, I want to confirm before quitting so that an accidental tap doesn't discard my progress.
- As a player who has just quit, I want to see a clear "Start New Game" button so that I can immediately begin a new game or browse other tabs.
- As a player, I want the quit option to be reachable from anywhere on the game screen (including while scrolled down through the scorecard).

## Functional Requirements
1. The game header shall display a menu icon (e.g., ☰ or ⚙️) that is visible whenever an active game is in progress.
2. Tapping the menu icon shall expand an options panel containing at least a "Quit Game" button.
3. Tapping "Quit Game" shall display a confirmation overlay with a Grandma quote drawn randomly from the `quit` pool and two actions: "Confirm Quit" and "Cancel".
4. Tapping "Cancel" shall dismiss the confirmation overlay and return the player to the active game without any state change.
5. Tapping "Confirm Quit" shall call `storage::clear_in_progress()`, reset all in-progress game signals, and transition the app to the idle/pre-game state.
6. The idle/pre-game screen shall be displayed on the Game tab and shall show a Grandma welcome quote (drawn from the `opening` quote pool) and a prominently placed "Start New Game" button.
7. Tapping "Start New Game" from the idle screen shall trigger the existing opening-quote → new game flow (i.e., behave identically to the current fresh-load path).
8. The tab bar shall remain visible on the idle/pre-game screen.
9. The menu icon shall not be visible (or shall be inert) when no game is active (i.e., on the idle screen, end-game overlay, or resume-prompt overlay).
10. The confirmation overlay shall be dismissable by tapping outside it or pressing Escape.
11. A `quit` pool of 25 Grandma quotes shall be present in `assets/grandma_quotes.json` and deserialized into `QuoteBank.quit`; the confirmation overlay shall select one at random each time it is shown.

## UI / UX Notes
- **Menu icon placement:** Top-right area of the game header, after the turn counter and roll pips. A small ☰ (hamburger) or ⋮ (vertical ellipsis) icon is recommended so it reads as "more options" without cluttering the header.
- **Options panel:** A small dropdown or slide-down panel anchored to the menu icon. Initially contains one item: "Quit Game" (with a destructive/red styling cue). The panel dismisses on outside tap.
- **Confirmation overlay:** Follows the visual pattern of the existing `ConfirmZero` component. Centered modal, semi-transparent backdrop, Grandma quote drawn randomly from the new `quit` pool in `grandma_quotes.json`. Two buttons: "Quit" (destructive) and "Keep Playing".
- **Idle screen:** Replaces the scorecard area when no active game exists. Shows a Grandma quote (from `opening` pool, selected once on quit) and a large "Start New Game" CTA button. The SIXZEE wordmark/header may remain for visual continuity.
- **Accessibility:** The menu icon must have an `aria-label="Game menu"`. The confirmation overlay must trap focus. Confirm and Cancel buttons must be keyboard-focusable.
- **Wireframe reference:** The game header is described in wireframes.md Screen 1 (header row). This feature extends that header section and introduces a new idle-state variant of Screen 1.

## Architecture Fit

### Existing modules/components touched
| File | Change |
|---|---|
| `src/app.rs` | Add `game_active: RwSignal<bool>` (or change `game_signal` to `RwSignal<Option<GameState>>`); wire new idle-screen conditional render |
| `src/state/mod.rs` | Expose `game_active` signal via context, or adjust `GameState` context to be `Option`-wrapped |
| `src/components/game_view.rs` | Render `GameMenu` in header; pass `on_quit_confirmed` callback down to it; conditionally show idle screen vs. active game |
| `src/storage.rs` | No changes needed — `clear_in_progress()` already exists |

### New modules/components to introduce
| File | Purpose |
|---|---|
| `src/components/game_menu.rs` | Expandable menu icon + options panel; emits `on_quit_requested` event |
| `src/components/confirm_quit.rs` | Confirmation overlay; emits `on_confirm` / `on_cancel`; reuses `ConfirmZero` visual pattern |
| `src/components/idle_screen.rs` | Pre-game screen rendered on Game tab when `game_active` is false; shows Grandma quote + "Start New Game" button |

### State shape changes
- Introduce a `game_active: RwSignal<bool>` signal at the App level (provided via context as a newtype `GameActive(pub RwSignal<bool>)` to avoid TypeId collisions).
- On quit confirmed: set `game_active` to `false`; on new game started: set it to `true`.
- Alternatively, wrapping `game_signal` as `RwSignal<Option<GameState>>` achieves the same goal with less new surface area — evaluate during implementation.

## Open Questions
1. **Quote pool for idle screen:** The idle screen should reuse the `opening` quote pool (selected once on quit-confirm). No new quote category needed.
2. **Menu icon style:** Will use ⋮ (U+22EE, vertical ellipsis) to avoid confusion with the ⚙️ Settings tab. The ellipsis reads as "more options" which is the correct affordance.
3. **Quit while Sixzee-bonus turn is active:** The quit flow is identical — discard everything. No special messaging needed.
4. **First-load with no saved game:** The idle screen only appears after an explicit quit. First load still auto-starts with the opening-quote overlay as today.

## Out of Scope / Future Work
- Saving DNF games to history.
- Undo last score placement (separate feature).
- Adding additional items to the game menu (e.g., "How to Play", "Report Bug").
- Modifying `ResumePrompt` behaviour — that flow already handles "Discard and Start New" correctly.

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan

### Files modified
| File | Change |
|---|---|
| `src/state/mod.rs` | Added `GameActive(pub RwSignal<bool>)` newtype |
| `src/state/quotes.rs` | Added `pub quit: Vec<String>` to `QuoteBank` + Tier 1 test |
| `src/app.rs` | Added `game_active: RwSignal<bool>` signal; provided as `GameActive` context |
| `src/components/mod.rs` | Added `confirm_quit`, `game_menu`, `idle_screen` module declarations |
| `src/components/game_view.rs` | Imported new components; added `show_confirm_quit` signal; wrapped view in `game_active` conditional; added `GameMenu` to header; wired quit callbacks |
| `src/components/confirm_quit.rs` | **New** — quit confirmation overlay with Grandma quote |
| `src/components/game_menu.rs` | **New** — ⋮ menu button + dropdown panel |
| `src/components/idle_screen.rs` | **New** — pre-game idle screen with "Start New Game" |
| `assets/grandma_quotes.json` | Added `quit` pool with 25 Grandma quit quotes |
| `style/main.css` | Added `.game-menu`, `.btn--danger`, `.overlay--quit`, `.idle-screen` styles |
| `tests/integration.rs` | 6 new browser integration tests for the quit flow |

### Key architectural decisions
- `GameActive(RwSignal<bool>)` newtype over `RwSignal<Option<GameState>>` — minimally invasive; all existing `game_signal` consumers untouched.
- `game_active = true` on first load — idle screen only appears after an explicit quit; first-launch experience unchanged.
- `GameMenu` dropdown uses a backdrop `<div>` for outside-click dismissal, consistent with `GrandmaQuoteOverlay`.
- Escape-key dismissal intentionally omitted (see Decisions Made).

### Deviations from Architecture Fit section
- No deviations. `RwSignal<Option<GameState>>` was the alternative evaluated and rejected. `GameActive` bool chosen as documented.

## Spec Changes
- **`doc/prd.md`**: Added 3 quit-game user stories (§4) and Requirement 59 (§6).
- **`doc/tech_spec.md`**: Added 3 new component files to layout; added `GameActive` context signal; documented `QuoteBank.quit` field.
- **`doc/wireframes.md`**: Updated game header (⋮ on all screen 1-3 header rows); added Screen 11 (Idle Screen) and Screen 12 (Quit Confirmation Overlay).
- **`doc/project_plan.md`**: Added Post-Milestone Features section with F-001 entry.

## Test Strategy

### Tests added
| Test | Tier | File | Coverage |
|---|---|---|---|
| `pick_quote_from_quit_shaped_pool_returns_member` | 1 | `src/state/quotes.rs` | `pick_quote` on quit-shaped pool |
| `game_menu_button_present_during_active_game` | 3 | `tests/integration.rs` | FR 1 — menu icon visible |
| `game_menu_panel_shows_on_click` | 3 | `tests/integration.rs` | FR 2 — panel opens with Quit Game item |
| `confirm_quit_overlay_shown_on_quit_tap` | 3 | `tests/integration.rs` | FR 3 — confirmation overlay appears |
| `cancel_quit_returns_to_active_game` | 3 | `tests/integration.rs` | FR 4 — cancel dismisses overlay, game intact |
| `confirm_quit_shows_idle_screen` | 3 | `tests/integration.rs` | FR 5+6+9 — quit → idle, no header, no menu |
| `start_new_game_from_idle_returns_to_game` | 3 | `tests/integration.rs` | FR 7 — idle → game header visible |

### Coverage gaps
- FR 8 (tab bar visible on idle) — waived; tab-bar visibility is CSS-driven and covered by existing tests.
- FR 10 (backdrop click cancels) — waived; JS click-propagation stoppage is hard to assert in headless browser; verified via smoke test.
- FR 11 quote text — not asserted in tests (selection is random); pool deserialization is validated by JSON validation in make.py.

## Decisions Made

### Decision: `GameActive` bool vs `RwSignal<Option<GameState>>`
**Chosen:** Separate `GameActive(RwSignal<bool>)` newtype alongside existing `RwSignal<GameState>`.
**Alternatives considered:** Changing `game_signal` type to `RwSignal<Option<GameState>>`, requiring updates to all 10+ consumers.
**Rationale:** Minimally invasive. All existing signal consumers remain untouched. A separate gating signal is idiomatic in this codebase.

### Decision: Escape key not implemented
**Chosen:** Backdrop-click to cancel only.
**Alternatives considered:** `keydown` event listener with `Closure` + manual cleanup.
**Rationale:** The `Send+Sync` constraint on `on_cleanup` prevents `gloo_events::EventListener` (lessons.md L10). Raw `Closure` approach adds complexity for a minor UX detail. Backdrop-click is sufficient for touch-first UX.

### Decision: ⋮ vertical ellipsis as menu icon
**Chosen:** U+22EE ⋮ vertical ellipsis.
**Alternatives considered:** ☰ hamburger (navigation connotation), ⚙️ gear (already used for Settings tab).
**Rationale:** ⋮ is the standard "more options" affordance on mobile; avoids confusion with the Settings tab.

### Decision: `game_active = true` on first load
**Chosen:** First launch is unaffected; idle screen only appears after an explicit quit.
**Alternatives considered:** Showing idle screen on first load (user explicitly starts a game).
**Rationale:** Preserves the current first-launch experience (direct opening-quote → game) which users of the existing app expect.

## Lessons / Highlights

### Leptos Reactive Gate Pattern
Wrapping an entire view branch in `{move || { if signal.get() { ... } else { ... } }}` is the idiomatic Leptos 0.8 way to conditionally render large sub-trees. The inner `view!` must call `.into_any()` on each branch for type unification. This pattern avoids introducing new context or re-architecting the component tree.

### Backdrop Div for Outside-Click Dismissal
Using a `position: fixed; inset: 0; z-index: N` backdrop `<div>` with `on:click=close` is a reliable, CSS-only approach to outside-click dismissal in Leptos. The panel sits at `z-index: N+1` above it. No JS event listeners needed. Used in both `GameMenu` (dropdown) and the existing `GrandmaQuoteOverlay` (full-screen overlay) — a pattern worth repeating for future overlays.

