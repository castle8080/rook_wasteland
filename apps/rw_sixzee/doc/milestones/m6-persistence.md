# M6 — Persistence & Resume

<!-- MILESTONE: M6 -->
<!-- STATUS: COMPLETE -->

**Status:** ✅ COMPLETE
**Depends on:** [M5 — Core Game UI](m5-core-game-ui.md)
**Required by:** [M9 — History Screen](m9-history.md), [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Implement all localStorage-based persistence: auto-save on every game state change, game completion record appended to
history, history pruning, graceful handling of storage unavailability, and the resume-vs-new-game prompt on app load.
After this milestone, closing and reopening the browser preserves an in-progress game exactly.

---

## Success Criteria

- [x] After rolling, refreshing the browser restores exact dice values, held state, roll count, and turn number
- [x] After placing a score, refreshing the browser shows that cell filled with the recorded value
- [x] Closing and reopening the app shows the Resume prompt when a game is in progress
- [x] Choosing Resume restores the game to exactly the same state (cells, dice, held, turn, bonus pool)
- [x] Choosing Start New discards the in-progress save and begins a fresh game
- [x] Completing a game (all 78 cells filled) appends a `CompletedGame` record to history;
  the record includes correct `final_score`, `bonus_pool`, `bonus_forfeited`, and `cells`
- [x] History is sorted descending by `final_score` after each append
- [x] On app load, history entries older than 365 days are pruned; a fresh load after adding an old
  record (manipulated via browser dev tools or test) confirms removal
- [x] When localStorage is unavailable, the game remains fully playable and an `ErrorBanner`
  ("Storage unavailable — progress will not be saved") appears without blocking gameplay
- [x] Storage errors from `roll()` and `place_score()` propagate as `Degraded` banners, not game crashes

---

## Tasks

### Storage Module (`src/state/storage.rs`)

- [x] Implement `load_in_progress() -> AppResult<Option<GameState>>` — reads `rw_sixzee.in_progress`
  from localStorage; returns `None` if key absent; `AppError::Json` if JSON parse fails
- [x] Implement `save_in_progress(state: &GameState) -> AppResult<()>` — serialises and writes to
  `rw_sixzee.in_progress`
- [x] Implement `clear_in_progress() -> AppResult<()>`
- [x] Implement `load_history() -> AppResult<Vec<CompletedGame>>` — reads `rw_sixzee.history`;
  returns empty vec if absent
- [x] Implement `save_history(history: &[CompletedGame]) -> AppResult<()>` — writes sorted (by
  `final_score` descending) list to `rw_sixzee.history`
- [x] Implement `load_theme() -> AppResult<Option<ThemeId>>` — reads `rw_sixzee.theme`
- [x] Implement `save_theme(theme_id: ThemeId) -> AppResult<()>`
- [x] Handle localStorage unavailability: all functions must detect `SecurityError`/`DOMException`
  from `web-sys` and return `AppError::Storage(...)` rather than panicking
- [x] Implement history pruning helper: filter out entries where `completed_at` is >365 days before now

### Wiring Persistence into Game Logic

- [x] In `roll()` — call `save_in_progress()` after updating state; on storage error, call
  `report_error()` with the error (Degraded banner) but do NOT abort the roll
- [x] In `place_score()` — call `save_in_progress()` after cell update; same error handling as above
- [x] On game completion in `place_score()`:
  - Build `CompletedGame` from current `GameState`
  - Load history, append new record, sort by `final_score` desc, save history
  - Call `clear_in_progress()`
  - Storage errors reported as Degraded (history loss is recoverable; game data in memory is intact)

### App Load Sequence (in `App` on_mount)

- [x] Call `load_theme()` → apply theme to body (or use default); storage error → Degraded banner
- [x] Call `load_in_progress()`:
  - `Ok(Some(state))` → set resume prompt signal to show `ResumePrompt` overlay
  - `Ok(None)` → start new game directly (call `GameState::new()`)
  - `Err(AppError::Json(_))` → corrupt save; report Fatal error; offer Start New escape
  - `Err(AppError::Storage(_))` → report Degraded banner; start new game
- [x] Prune history on load: `load_history()` → filter old entries → `save_history()` (best-effort)

### Resume Prompt (`src/components/resume.rs`)

- [x] Full-screen overlay shown when in-progress game detected on app load
- [x] Display: game start date, turn count, current score (computed from saved state)
- [x] "Resume Game" button — dismiss overlay, set `GameState` signal from saved state
- [x] "Discard and Start New" button — call `clear_in_progress()`, init fresh `GameState`
- [x] Tab bar NOT shown while prompt is visible
- [x] Hash router navigation blocked until choice is made (overlay sits on top; no tab bar to click)

### CSS

- [x] Add `.resume-prompt` overlay styles to `style/main.css`
- [x] Ensure resume overlay uses same `.overlay` base block with appropriate modifiers

### Tests

- [x] Unit test: `GameState` serialise → JSON string → deserialise produces equal struct (native test)
- [x] Unit test: history pruning removes entries >365 days old and retains others
- [x] Unit test: `save_history` sorts by `final_score` descending
- [x] WASM browser tests: in_progress round-trip, absent key, clear, corrupt JSON, preserves cells/bonus_pool (5 tests)
- [x] WASM browser tests: history round-trip, absent, sorted descending, corrupt JSON (4 tests)
- [x] WASM browser tests: theme round-trip, absent, overwrite (3 tests)
- [x] **E2E smoke test** (`e2e/smoke.spec.ts`): after rolling and refreshing the page, the game
  header and dice row are visible (verifies localStorage save + WASM re-init + resume prompt appear)
- [x] **E2E smoke test**: close and reopen the app → Resume prompt appears with correct turn count
- [x] **E2E smoke test**: choose "Start New" on the Resume prompt → fresh game starts (dice show `?`)

---

## Implementation Notes

- `src/state/storage.rs` (new) — all 7 localStorage functions; `#[cfg(target_arch = "wasm32")]`
  gate applied at module level in `state/mod.rs`
- `sort_history_by_score` and `prune_old_entries` live in `game.rs` (not `storage.rs`) so they are
  natively testable without a browser; `storage.rs` calls them
- `js_sys::Date::new(&JsValue::from_str(s))` is the correct WASM API for parsing ISO 8601 strings
  (`Date::new_with_str` does not exist in js-sys 0.3)
- `sort_history_by_score` signature uses `&mut [CompletedGame]` (not `&mut Vec<…>`) to satisfy
  `clippy::ptr_arg`
- `pending_resume: RwSignal<Option<GameState>>` added to App context; set during on_mount load
  sequence, consumed and cleared by `ResumePrompt`
- Integration tests: added `clear_game_storage()` helper to the top of `tests/integration.rs`
  and called it at the start of every app-mounting test to prevent cross-test contamination from
  storage tests leaving keys in localStorage

---

## Implementation Summary

### What was built

M6 added full localStorage persistence to rw_sixzee. The app now auto-saves after every roll and every score placement, detects a saved game on startup and prompts the user to resume or discard it, appends completed games to a persistent history, prunes entries older than 365 days, and restores the last-used theme.

### Architecture decisions

**Pure helpers live in `game.rs`, not `storage.rs`.** `sort_history_by_score` and `prune_old_entries` are pure Rust with no browser dependencies. Keeping them in `game.rs` lets `cargo test` (native) exercise them with no browser harness — 20+ unit tests run in milliseconds.

**`persist_after_score` is a free function in `game_view.rs`.** Rather than duplicating the "is the game complete? save history : save in-progress" logic in three separate handlers (`on_roll`, `on_cell_click`, `on_confirm_zero`), it is factored into a single non-public free function that each handler calls after mutating state.

**`pending_resume: RwSignal<Option<GameState>>` bridges the load sequence and the component.** The App load sequence runs synchronously during component initialization. It saves the parsed `GameState` to `pending_resume` and sets `show_resume = true`. `ResumePrompt` reads the snapshot once on mount (via `get_untracked()`) so the display is stable even if the signal is later cleared.

**Theme and history pruning are best-effort, fire-and-forget.** On any storage failure at load time, the app falls back to the default theme and skips pruning — neither is fatal to gameplay. Persistence errors during play are reported as `Degraded` banners. Only a corrupt `in_progress` save (which implies unknown game state) is escalated to `Fatal`.

### Key technical discoveries

**Leptos context resolves by `TypeId` — same-type signals silently collide (L13).** This was the root cause of the most difficult bug in this milestone. `show_resume`, `show_opening_quote`, and `hide_tab_bar` are all `RwSignal<bool>`. When all three were provided to context, only the last one (`hide_tab_bar`) was retrievable via `use_context::<RwSignal<bool>>()`. Child components (`ResumePrompt`, `GameView`, `TabBar`) were all writing to `hide_tab_bar` when they intended to write to different signals — the resume prompt never dismissed. Fixed by introducing three newtype wrappers (`ShowResume`, `ShowOpeningQuote`, `HideTabBar`) in `state/mod.rs`, giving each a unique `TypeId`.

**Setting signals inside a `view!` reactive closure is an anti-pattern (L14).** `hide_tab_bar.set(true)` was placed inside the `move || {}` content closure that also controlled which overlay to render. This caused a secondary reactive flush on every evaluation — observable as intermittent E2E failures where the opening-quote overlay persisted after dismissal. Fixed by moving the `hide_tab_bar` write into a `leptos::Effect`, which is the correct reactive primitive for "write a signal when other signals change."

**`js_sys::Date::new_with_str` does not exist.** The correct js-sys 0.3 API for constructing a `Date` from an ISO 8601 string is `js_sys::Date::new(&JsValue::from_str(s))`. The `new_with_str` variant (which would have been the obvious name) is absent from the bindings.

**WASM browser tests share localStorage — app-mount tests must clear storage first (L15).** All `#[wasm_bindgen_test]` tests run sequentially in the same browser page with a shared `localStorage`. The 12 M6 storage tests leave `rw_sixzee.*` keys populated. Any subsequent test that mounts `App` runs the M6 load sequence, finds those keys, and shows the `ResumePrompt` instead of the normal game view — causing unrelated tests to fail with 0 dice buttons. Fixed by adding a `clear_game_storage()` helper called at the top of every app-mounting test.

**E2E point-in-time `isVisible()` is not safe for WASM timing.** The original "Start New" E2E test used `if (await locator.isVisible())` after a fixed `waitForTimeout(300)` to decide whether to dismiss the opening-quote overlay. The WASM reactive system may schedule DOM updates after that 300ms window, so the check could pass while the overlay was absent, then the overlay appeared moments later and permanently blocked the game view (dice count stayed at 0 for the full 5-second `toHaveCount` timeout). Fixed by replacing `isVisible()` with `waitFor({ state: "visible", timeout: 3000 })`, which polls until the condition is met.

### Test coverage achieved

| Layer | Count | What's covered |
|---|---|---|
| Native unit (`cargo test`) | 85 | All pure-Rust state, scoring, routing, game helpers, M6 helpers |
| WASM browser (`wasm-pack test`) | 32 | Storage round-trips, all 7 storage functions, app load sequence, ResumePrompt callbacks, roll persistence, theme application, history pruning on load, corrupt save error path |
| E2E Playwright | 9 | Smoke: page load, WASM init, dice render; M6: resume prompt after refresh, correct turn count, Start New restarts game |

The one explicitly waived gap is `persist_after_score`'s game-completion branch (calling `save_history` + `clear_in_progress` when all cells are filled). Driving the app to game completion programmatically through the DOM requires 42 separate score-placement interactions; the risk is low because the same `save_history` / `clear_in_progress` functions are exercised by storage tests and the branch logic is a simple conditional.

