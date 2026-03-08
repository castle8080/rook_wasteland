# M6 — Persistence & Resume

<!-- MILESTONE: M6 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** [M5 — Core Game UI](m5-core-game-ui.md)
**Required by:** [M9 — History Screen](m9-history.md), [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Implement all localStorage-based persistence: auto-save on every game state change, game completion record appended to
history, history pruning, graceful handling of storage unavailability, and the resume-vs-new-game prompt on app load.
After this milestone, closing and reopening the browser preserves an in-progress game exactly.

---

## Success Criteria

- [ ] After rolling, refreshing the browser restores exact dice values, held state, roll count, and turn number
- [ ] After placing a score, refreshing the browser shows that cell filled with the recorded value
- [ ] Closing and reopening the app shows the Resume prompt when a game is in progress
- [ ] Choosing Resume restores the game to exactly the same state (cells, dice, held, turn, bonus pool)
- [ ] Choosing Start New discards the in-progress save and begins a fresh game
- [ ] Completing a game (all 78 cells filled) appends a `CompletedGame` record to history;
  the record includes correct `final_score`, `bonus_pool`, `bonus_forfeited`, and `cells`
- [ ] History is sorted descending by `final_score` after each append
- [ ] On app load, history entries older than 365 days are pruned; a fresh load after adding an old
  record (manipulated via browser dev tools or test) confirms removal
- [ ] When localStorage is unavailable, the game remains fully playable and an `ErrorBanner`
  ("Storage unavailable — progress will not be saved") appears without blocking gameplay
- [ ] Storage errors from `roll()` and `place_score()` propagate as `Degraded` banners, not game crashes

---

## Tasks

### Storage Module (`src/state/storage.rs`)

- [ ] Implement `load_in_progress() -> AppResult<Option<GameState>>` — reads `rw_sixzee.in_progress`
  from localStorage; returns `None` if key absent; `AppError::Json` if JSON parse fails
- [ ] Implement `save_in_progress(state: &GameState) -> AppResult<()>` — serialises and writes to
  `rw_sixzee.in_progress`
- [ ] Implement `clear_in_progress() -> AppResult<()>`
- [ ] Implement `load_history() -> AppResult<Vec<CompletedGame>>` — reads `rw_sixzee.history`;
  returns empty vec if absent
- [ ] Implement `save_history(history: &[CompletedGame]) -> AppResult<()>` — writes sorted (by
  `final_score` descending) list to `rw_sixzee.history`
- [ ] Implement `load_theme() -> AppResult<Option<ThemeId>>` — reads `rw_sixzee.theme`
- [ ] Implement `save_theme(theme_id: ThemeId) -> AppResult<()>`
- [ ] Handle localStorage unavailability: all functions must detect `SecurityError`/`DOMException`
  from `web-sys` and return `AppError::Storage(...)` rather than panicking
- [ ] Implement history pruning helper: filter out entries where `completed_at` is >365 days before now

### Wiring Persistence into Game Logic

- [ ] In `roll()` — call `save_in_progress()` after updating state; on storage error, call
  `report_error()` with the error (Degraded banner) but do NOT abort the roll
- [ ] In `place_score()` — call `save_in_progress()` after cell update; same error handling as above
- [ ] On game completion in `place_score()`:
  - Build `CompletedGame` from current `GameState`
  - Load history, append new record, sort by `final_score` desc, save history
  - Call `clear_in_progress()`
  - Storage errors reported as Degraded (history loss is recoverable; game data in memory is intact)

### App Load Sequence (in `App` on_mount)

- [ ] Call `load_theme()` → apply theme to body (or use default); storage error → Degraded banner
- [ ] Call `load_in_progress()`:
  - `Ok(Some(state))` → set resume prompt signal to show `ResumePrompt` overlay
  - `Ok(None)` → start new game directly (call `GameState::new()`)
  - `Err(AppError::Json(_))` → corrupt save; report Fatal error; offer Start New escape
  - `Err(AppError::Storage(_))` → report Degraded banner; start new game
- [ ] Prune history on load: `load_history()` → filter old entries → `save_history()` (best-effort)

### Resume Prompt (`src/components/resume.rs`)

- [ ] Full-screen overlay shown when in-progress game detected on app load
- [ ] Display: game start date, turn count, current score (computed from saved state)
- [ ] "Resume Game" button — dismiss overlay, set `GameState` signal from saved state
- [ ] "Discard and Start New" button — call `clear_in_progress()`, init fresh `GameState`
- [ ] Tab bar NOT shown while prompt is visible
- [ ] Hash router navigation blocked until choice is made (overlay sits on top; no tab bar to click)

### CSS

- [ ] Add `.resume-prompt` overlay styles to `style/main.css`
- [ ] Ensure resume overlay uses same `.overlay` base block with appropriate modifiers

### Tests

- [ ] Unit test: `GameState` serialise → JSON string → deserialise produces equal struct (native test)
- [ ] Unit test: history pruning removes entries >365 days old and retains others
- [ ] Unit test: `save_history` sorts by `final_score` descending

---

## Notes & Risks

- **localStorage in private browsing:** `window.localStorage` property access throws a `SecurityError`
  in some browsers when in private mode. Wrap the initial `window.local_storage()` call in a try/catch
  and short-circuit all storage functions if unavailable, returning `AppError::Storage`.
- **JSON corruption:** If `rw_sixzee.in_progress` contains corrupted JSON (e.g. partial write due to
  crash), `serde_json::from_str` returns an error. Treat this as Fatal — it's an unexpected failure.
  Offer the user a clear escape via the Fatal overlay's "Start New Game" action.
- **History sort order:** History is always stored sorted descending. On load, no re-sort is required
  unless entries are added. Always sort before saving to keep the invariant.
- **`started_at` parsing:** Use `js_sys::Date::now()` (milliseconds) for `started_at` generation in
  `GameState::new()` and convert to ISO 8601 string. For pruning, parse back via `js_sys::Date`
  constructor from the string. In native tests, you can use a hardcoded timestamp string.
