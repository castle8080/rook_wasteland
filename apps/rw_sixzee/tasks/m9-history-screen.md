# Task M9: History Screen

**Milestone:** M9 — History Screen
**Status:** 🔄 In Progress

## Restatement

Implement the History screen and History Detail view so players can review completed
games. The History list (`src/components/history.rs`) loads `Vec<CompletedGame>` from
`localStorage` on mount, renders a ranked table with medal emojis for top 3, and
navigates to `#/history/:id` on "View →" click. The History Detail view
(`src/components/history_detail.rs`) accepts an `id` prop, looks up the game in
storage, renders a read-only scorecard via a new `ScorecardReadOnly` component, and
provides a back button. The `Scorecard` component gains a sibling `ScorecardReadOnly`
that accepts a static cell snapshot rather than reactive signals. The end-of-game
"View Full Scorecard" button (already wired in M5) lands on the detail view.
Out of scope: editing history, pagination, search/filter.

## Design

### Data flow

- **HistoryView mount** → `storage::load_history()` (sync localStorage read) →
  `Vec<CompletedGame>` (pre-sorted descending by `final_score`) → rendered table rows.
- **"View →" click** → `route.set(Route::HistoryDetail { id })` + `navigate(&dest)` →
  hashchange updates URL → App re-renders with `HistoryDetail { id }`.
- **HistoryDetail mount** → `storage::load_history()` → find by `id` →
  `ScorecardReadOnly { cells, bonus_pool, bonus_forfeited }`.
- **Back button** → `route.set(Route::History)` + `navigate(&Route::History)`.

### Function / type signatures

```rust
// scorecard.rs — new sibling component
/// Read-only 6-column × 13-row scorecard snapshot for the History Detail view.
#[component]
pub fn ScorecardReadOnly(
    cells: [[Option<u8>; ROW_COUNT]; 6],
    bonus_pool: u32,
    bonus_forfeited: bool,
) -> impl IntoView

// history.rs — replace stub
#[component]
pub fn HistoryView() -> impl IntoView

// history_detail.rs — new file
#[component]
pub fn HistoryDetail(id: String) -> impl IntoView

// shared helper (history.rs)
fn format_date(iso: &str) -> String       // "2026-03-07T…" → "Mar 7, 2026"
fn rank_display(rank: usize) -> String    // 1→"🥇", 2→"🥈", 3→"🥉", N→"N"
fn row_class(rank: usize) -> &'static str // BEM modifier for medal rows
```

### Edge cases

- `load_history()` returns `Err` (private browsing / corrupt JSON) → `unwrap_or_default()`
  yields empty vec → shows empty-state message; error is silently swallowed (non-critical read).
- Empty history → show `<p class="history-list__empty">` message.
- `HistoryDetail` with unknown `id` (e.g. deep-linked to deleted game) → "Game not found" + back button.
- `format_date` on malformed ISO string → falls back to raw ISO string.
- `rank_display` for N ≥ 4 → plain numeric string.

### Integration points

| File | Change |
|---|---|
| `src/components/scorecard.rs` | Add `ScorecardReadOnly` component; extend imports |
| `src/components/history.rs` | Replace 8-line stub with full `HistoryView` |
| `src/components/history_detail.rs` | New file: `HistoryDetail` component |
| `src/components/mod.rs` | Add `pub mod history_detail` |
| `src/app.rs` | Import `HistoryDetail`; replace placeholder in `Route::HistoryDetail` arm |
| `style/main.css` | Add `.history-list*` and `.history-detail*` rules |
| `tests/integration.rs` | Add history list + detail integration tests |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `load_history()` called on every mount — fresh data on each nav, but if called twice in rapid succession the list may briefly diverge | Acceptable: components are destroyed/remounted on route change; no double-mount scenario |
| Simplicity | Could reuse `Scorecard` with a `readonly` bool prop instead of a separate component | Separate component avoids context coupling; spec explicitly allows this approach |
| Coupling | `HistoryDetail` directly calls `storage::load_history()` rather than receiving the game as a prop | Acceptable: avoids threading state through the router; aligns with existing on-demand load pattern |
| Performance | Full history list deserialized on every visit to History tab | Acceptable: history is small (<365 entries); localStorage read is sync and fast |
| Testability | `format_date` and `rank_display` are pure functions → trivially unit-testable | ✅ Covered by `#[cfg(test)]` block in history.rs |

## Implementation Notes

- `end_game.rs` already calls `route.set()` + `navigate()` for "View Full Scorecard" — pattern confirmed and reused in `HistoryView`.
- `bonus_pool_label` used in `ScorecardReadOnly` footer for live Sixzee-fill count display.
- CSS follows existing BEM conventions; medal tints use `color-mix` against `--color-surface`.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `format_date` happy path | 1 | ✅ | unit test in history.rs |
| `format_date` malformed input | 1 | ✅ | unit test |
| `rank_display` medals (1–3) | 1 | ✅ | unit test |
| `rank_display` numeric (N≥4) | 1 | ✅ | unit test |
| `HistoryView` empty state DOM | 3 | ✅ | integration.rs |
| `HistoryView` list rows DOM | 3 | ✅ | integration.rs |
| `HistoryView` medal class on top-3 row | 3 | ✅ | integration.rs |
| `HistoryDetail` unknown id → "not found" | 3 | ✅ | integration.rs |
| `HistoryDetail` known id → scorecard cells | 3 | ✅ | integration.rs |
| `ScorecardReadOnly` renders filled cells | 3 | ✅ | covered by HistoryDetail test |
| Back button navigates to #/history | 3 | ✅ | integration.rs |

## Test Results

(filled after Phase 6)

## Review Notes

(filled after Phase 7)

## Callouts / Gotchas

(filled after Phase 10)
