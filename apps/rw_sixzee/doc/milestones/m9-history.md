# M9 — History Screen

<!-- MILESTONE: M9 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** [M6 — Persistence & Resume](m6-persistence.md)
**Required by:** [M10 — Polish & Mobile](m10-polish-mobile.md)

---

## Overview

Implement the History screen and History Detail view. Players can see all completed games sorted by score, with gold /
silver / bronze medals on the top 3, and drill into any game to view its full 6-column scorecard snapshot in read-only
mode. Navigation integrates with the hash router (`#/history` and `#/history/:id`).

---

## Success Criteria

- [ ] Navigating to History tab shows a list of all completed games sorted by final score descending
- [ ] Each row displays: rank (🥇/🥈/🥉 for top 3, then numeric), date, final score, Sixzee bonus pool amount
- [ ] Top 3 entries use medal emoji for rank; entries 4+ show numeric rank
- [ ] Clicking "View →" on any row navigates to `#/history/:id` and renders that game's scorecard
- [ ] The scorecard snapshot is read-only: cells show final values, no dice, no roll button, no previews
- [ ] Scorecard snapshot layout is identical to the active game scorecard (same BEM classes, same column structure)
- [ ] Navigating back (via `[ ← History ]` button) returns to the History list at `#/history`
- [ ] If no completed games exist, an empty-state message is shown ("No completed games yet. Finish your first game!")
- [ ] "View Full Scorecard" from the end-of-game summary overlay (M5) correctly navigates to the
  newly-saved game's detail view
- [ ] Games pruned (>365 days old) do not appear in the list

---

## Tasks

### History List (`src/components/history.rs`)

- [ ] Load `Vec<CompletedGame>` from context (provided by `App` on_mount from `load_history()`)
- [ ] Render as a table/list sorted descending by `final_score` (already sorted from storage)
- [ ] Each row: rank, formatted date (`completed_at` ISO string → human-readable), `final_score`,
  `bonus_pool` (shown as `+N`), `[ View → ]` button
- [ ] Rank rendering: 1 = 🥇, 2 = 🥈, 3 = 🥉, N≥4 = plain number
- [ ] Empty state: `<p class="history-list__empty">No completed games yet. Finish your first game!</p>`
- [ ] Clicking `[ View → ]` calls `navigate(Route::HistoryDetail { id: game.id.clone() })`

### History Detail (`src/components/history_detail.rs`)

- [ ] Accept `id: String` prop (from router)
- [ ] Look up `CompletedGame` from context signal by id
- [ ] If not found: show "Game not found" with `[ ← History ]` back button
- [ ] Render header: `[ ← History ]` back button, date, `Final Score: N`
- [ ] Render read-only scorecard using the same `Scorecard` component from M5 but with:
  - No dice state / preview computation (pass zeros or use a variant prop)
  - No cell click handlers
  - `cells: [[Option<u8>; 13]; 6]` from `CompletedGame.cells`
- [ ] Show Sixzee bonus pool box (with forfeiture label if `bonus_forfeited = true`)
- [ ] Show grand total
- [ ] `[ ← History ]` back button calls `navigate(Route::History)`
- [ ] Tab bar visible and functional on this screen

### Scorecard Reuse for Read-Only View

- [ ] Extend `Scorecard` component to accept a `readonly: bool` prop (or use a separate `ScorecardReadOnly`
  wrapper component that suppresses click handlers and preview computation)
- [ ] Ensure the layout, spacing, and BEM classes are identical between active and read-only views

### Router Integration

- [ ] Confirm `Route::HistoryDetail { id }` is handled in `App`'s conditional render
- [ ] Pass `id` to `HistoryDetail` component
- [ ] `navigate(Route::History)` from back button updates hash to `#/history`

### CSS

- [ ] Add `.history-list`, `.history-list__header`, `.history-list__row`, `.history-list__row--gold`,
  `.history-list__row--silver`, `.history-list__row--bronze`, `.history-list__empty`
- [ ] Medal row variants have subtle background tint corresponding to medal colour
- [ ] `.history-detail__header` with back button layout (left-aligned arrow + centred title)
- [ ] **E2E smoke test** (`e2e/smoke.spec.ts`): after a game completes in E2E, navigate to `#/history`,
  verify at least one `.history-list__row` is visible with a non-zero score; click "View →",
  verify history detail loads and shows filled scorecard cells

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
