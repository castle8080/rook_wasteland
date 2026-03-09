# M9 — History Screen

<!-- MILESTONE: M9 -->
<!-- STATUS: DONE -->

**Status:** ✅ Done
**Depends on:** [M6 — Persistence & Resume](m6-persistence.md)
**Required by:** [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Implement the History screen and History Detail view. Players can see all completed games sorted by score, with gold /
silver / bronze medals on the top 3, and drill into any game to view its full 6-column scorecard snapshot in read-only
mode. Navigation integrates with the hash router (`#/history` and `#/history/:id`).

---

## Success Criteria

- [x] Navigating to History tab shows a list of all completed games sorted by final score descending
- [x] Each row displays: rank (🥇/🥈/🥉 for top 3, then numeric), date, final score, Sixzee bonus pool amount
- [x] Top 3 entries use medal emoji for rank; entries 4+ show numeric rank
- [x] Clicking "View →" on any row navigates to `#/history/:id` and renders that game's scorecard
- [x] The scorecard snapshot is read-only: cells show final values, no dice, no roll button, no previews
- [x] Scorecard snapshot layout is identical to the active game scorecard (same BEM classes, same column structure)
- [x] Navigating back (via `[ ← History ]` button) returns to the History list at `#/history`
- [x] If no completed games exist, an empty-state message is shown ("No completed games yet. Finish your first game!")
- [x] "View Full Scorecard" from the end-of-game summary overlay (M5) correctly navigates to the
  newly-saved game's detail view
- [x] Games pruned (>365 days old) do not appear in the list

---

## Tasks

### History List (`src/components/history.rs`)

- [x] Load `Vec<CompletedGame>` from context (provided by `App` on_mount from `load_history()`)
- [x] Render as a table/list sorted descending by `final_score` (already sorted from storage)
- [x] Each row: rank, formatted date (`completed_at` ISO string → human-readable), `final_score`,
  `bonus_pool` (shown as `+N`), `[ View → ]` button
- [x] Rank rendering: 1 = 🥇, 2 = 🥈, 3 = 🥉, N≥4 = plain number
- [x] Empty state: `<p class="history-list__empty">No completed games yet. Finish your first game!</p>`
- [x] Clicking `[ View → ]` calls `navigate(Route::HistoryDetail { id: game.id.clone() })`

### History Detail (`src/components/history_detail.rs`)

- [x] Accept `id: String` prop (from router)
- [x] Look up `CompletedGame` from context signal by id
- [x] If not found: show "Game not found" with `[ ← History ]` back button
- [x] Render header: `[ ← History ]` back button, date, `Final Score: N`
- [x] Render read-only scorecard using the same `Scorecard` component from M5 but with:
  - No dice state / preview computation (pass zeros or use a variant prop)
  - No cell click handlers
  - `cells: [[Option<u8>; 13]; 6]` from `CompletedGame.cells`
- [x] Show Sixzee bonus pool box (with forfeiture label if `bonus_forfeited = true`)
- [x] Show grand total
- [x] `[ ← History ]` back button calls `navigate(Route::History)`
- [x] Tab bar visible and functional on this screen

### Scorecard Reuse for Read-Only View

- [x] Extend `Scorecard` component to accept a `readonly: bool` prop (or use a separate `ScorecardReadOnly`
  wrapper component that suppresses click handlers and preview computation)
- [x] Ensure the layout, spacing, and BEM classes are identical between active and read-only views

### Router Integration

- [x] Confirm `Route::HistoryDetail { id }` is handled in `App`'s conditional render
- [x] Pass `id` to `HistoryDetail` component
- [x] `navigate(Route::History)` from back button updates hash to `#/history`

### CSS

- [x] Add `.history-list`, `.history-list__header`, `.history-list__row`, `.history-list__row--gold`,
  `.history-list__row--silver`, `.history-list__row--bronze`, `.history-list__empty`
- [x] Medal row variants have subtle background tint corresponding to medal colour
- [x] `.history-detail__header` with back button layout (left-aligned arrow + centred title)
- [x] ~~**E2E smoke test** (`e2e/smoke.spec.ts`)~~ → replaced with wasm-pack browser integration tests
  (see Implementation Summary below)

---

## Notes & Risks

- **History state freshness:** The history list reads from a signal that `App` populates on_mount and
  updates when a game completes. Ensure the `RwSignal<Vec<CompletedGame>>` is updated by M6's game
  completion path so the History screen reactively shows new entries.
- **Date formatting:** `completed_at` is an ISO 8601 string. Convert to a human-readable format
  (e.g. "Mar 7, 2026") in the component. Use `js_sys::Date` for parsing or a simple string split —
  no external date library needed.
- **Scorecard readonly reuse:** The active scorecard has click handlers and preview logic tightly
  wired to `RwSignal<GameState>`. The cleanest approach is a `readonly` prop that skips those code
  paths and accepts a plain `cells` array, rather than duplicating the entire component.

---

## Implementation Summary

**Completed:** 2026-03-09 · 56 browser tests passing (up from 43 before M9)

### Key Design Decisions

**`ScorecardReadOnly` as a separate component (not a `readonly` prop on `Scorecard`)**
The original spec suggested a `readonly: bool` prop. During implementation it became clear
that `Scorecard` reads `game_signal`, `score_preview`, and `grand_total` directly from context
via `use_context`. Gating those reads behind a prop would require threading `Option<Signal<T>>`
through every internal closure, making the active path noisier. A dedicated `ScorecardReadOnly`
component accepting `cells`, `bonus_pool`, and `bonus_forfeited` as plain props is completely
decoupled from game context and is simpler to reason about.

**History loaded fresh on every mount (not a context signal)**
The spec note suggested an `RwSignal<Vec<CompletedGame>>` in context updated by the completion
path. In practice, Leptos destroys and remounts `HistoryView` every time the user switches to
the History tab, so calling `storage::load_history()` on mount is sufficient and avoids a
stale-signal class of bug. No signal wiring was added to `App`.

**`format_date` duplicated in `history.rs` and `history_detail.rs`**
Deliberately kept private in each module rather than creating a shared `util` module.
The function is four lines and the duplication is preferable to introducing a new module
boundary that would need to be maintained across future milestones.

**Medal tints via `color-mix`**
`color-mix(in srgb, #f6d860 18%, var(--color-surface))` produces a subtle gold tint that
adapts automatically to all 6 themes (light and dark) without hardcoding per-theme rules.
Same technique used for silver (`#c0c0c0`) and bronze (`#cd7f32`).

**End-game navigation test strategy: pre-seeded state, not E2E**
The "View Full Scorecard" button in `end_game.rs` was the last untested success criterion.
A full 78-turn UI simulation would take ~40–60 seconds. Instead: pre-seed a fully-complete
`GameState` in `in_progress` storage (triggering the resume prompt) AND the matching
`CompletedGame` in `history` storage. Resume → EndGame overlay renders immediately
(no turns played) → click button → assert `history-detail` renders. Total test time: ~1 second.

### Files Changed

| File | Change |
|---|---|
| `src/components/history.rs` | Full implementation (replaced 8-line stub) |
| `src/components/history_detail.rs` | New file |
| `src/components/scorecard.rs` | Added `ScorecardReadOnly` component |
| `src/components/mod.rs` | Registered `history_detail` module |
| `src/app.rs` | Wired `Route::HistoryDetail` arm to `<HistoryDetail>` |
| `style/main.css` | History list + detail CSS (`.history-list*`, `.history-detail*`) |
| `tests/integration.rs` | 13 new browser tests (12 M9 + 1 end-game navigation gap) |
| `doc/milestones/m9-history.md` | This file |

### Test Coverage

13 new browser integration tests added across three commits:

- **Initial M9 (5 tests):** empty state, list renders rows, detail not-found, detail renders scorecard, back button
- **Coverage audit follow-up (7 tests):** "View →" forward navigation, read-only guarantee (no dice/roll/preview), all 3 medal classes, 4th-place no-medal, score sort order, score/bonus text content, header date/score
- **End-game navigation gap (1 test):** `end_game_view_full_scorecard_navigates_to_history_detail`
