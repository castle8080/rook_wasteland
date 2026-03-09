# Task M10: Polish & Mobile

**Milestone:** M10 — Polish & Mobile
**Status:** ✅ Done

---

## Restatement

M10 is the final milestone for rw_sixzee. There are no new features; the goal is to
ensure every existing feature is correct on mobile, accessible to assistive technology,
handles edge cases gracefully, and is covered by a comprehensive test suite.

The audit revealed most heavy infrastructure is already in place (overlays are full-screen
`position:fixed`, mobile CSS breakpoint exists, dice already have `aria-label`). The gaps
are targeted:

1. **Responsive labels** — Scorecard row labels are full English names ("3 of a Kind") with
   no mobile-friendly abbreviation. Need dual-span CSS toggle: long name on ≥600px, short
   name ("3K") on ≤599px. Also expand the existing 599px block with full-width buttons and
   touch-target enforcement for scorecard cells.

2. **Tab bar hides for ConfirmZero** — The `hide_tab_bar` Effect in `app.rs` covers the
   opening-quote overlay, resume prompt, and grandma panel — but NOT the zero-score
   confirmation overlay. The per-spec requirement is that the tab bar hides during the
   zero-score confirmation. Fix: promote `pending_zero` to a context `RwSignal` (via a
   `PendingZero` newtype) so `app.rs` can include it in the Effect.

3. **Accessibility** — Four overlays (EndGame, Resume, ConfirmZero, ConfirmQuit) are missing
   `role="dialog"` + `aria-modal="true"`. Tab bar buttons lack `aria-current="page"`.
   Scorecard cells have no `aria-label` (they need one describing value/state per cell).

4. **Tests** — Three spec-required tests are missing: mini-game test (78 programmatic turns,
   assert `is_game_complete` + `grand_total`), DP table sanity (wasm-pack), and new browser
   integration tests for the accessibility and tab-bar-hide changes. Two E2E tests:
   pre-seeded full-game completion, and mobile viewport no-overflow.

**Out of scope:** Manual verification checklist (MT items), Ask Grandma worker round-trip
test (requires serving worker files from test origin — waived), iOS/Android device testing
(manual, not automatable).

---

## Design

### Data flow

**PendingZero promotion:**
- Currently: `pending_zero: RwSignal<Option<(usize, usize)>>` is local to `GameView`.
- Change: create signal in `app.rs`, provide as `PendingZero` context (same newtype pattern
  as `GameActive`, `ShowResume`). `GameView` reads it from context via `use_context`.
- `app.rs` `hide_tab_bar` Effect gains one dependency:
  `hide_tab_bar.set(quote_visible || show_resume.get() || grandma_open || pending_zero.get().is_some())`
- `ConfirmZero` loses the explicit `hide_tab_bar.set(false)` calls (tab bar is managed
  entirely by the Effect; when `pending_zero` returns to `None`, Effect fires and unsets).

**Abbreviated row labels:**
- `scoring.rs`: add `pub const ROW_LABELS_SHORT: [&str; ROW_COUNT]`
- `scorecard.rs` `cell_view` row `<td>`: render two spans with BEM modifier classes.
- `style/main.css`: `.scorecard__label--short { display:none }` at desktop;
  `@media (max-width:599px)` toggles to show short, hide long.
- Both `Scorecard` and `ScorecardReadOnly` need the same dual-span treatment.

**Scorecard cell aria-label:**
- Added to `cell_view` via a reactive `move || format!(...)` closure reading
  `game_signal` and `score_preview`.
- Format: `"{row_label}, Column {col+1}: {state_description}"` where state_description is
  one of: `"empty, click to score"` / `"preview {n} points, click to score"` / `"scored {n}"`.

### Function / type signatures

```rust
// src/state/scoring.rs — new constant
/// Abbreviated row labels for mobile display (≤599px).
pub const ROW_LABELS_SHORT: [&str; ROW_COUNT] = [
    "1s", "2s", "3s", "4s", "5s", "6s",
    "3K", "4K", "FH", "SS", "LS", "6Z", "CH",
];

// src/state/mod.rs — new newtype
/// Newtype for the zero-score confirmation signal — `Some((col,row))` while
/// the ConfirmZero overlay is shown.
#[derive(Clone, Copy)]
pub struct PendingZero(pub RwSignal<Option<(usize, usize)>>);
```

### Edge cases

- `pending_zero` becomes `Some` then immediately `None` (fast tap + cancel): Effect fires
  twice; final value is `None`, tab bar shows. Correct.
- Mobile viewport exactly 600px: falls into desktop mode (≥600px threshold). Labels stay long.
- Scorecard cell aria-label when `score_preview` is unrolled (all dice None): preview = 0,
  described as "empty, click to score" since `rolls_used == 0` implies no preview.
- `ScorecardReadOnly`: no aria-label on cells needed (read-only, not interactive). Cells
  already have `scorecard__cell--filled` and text content which screen readers will read.

### Integration points

| File | Change |
|---|---|
| `src/state/scoring.rs` | Add `ROW_LABELS_SHORT` |
| `src/state/mod.rs` | Add `PendingZero` newtype |
| `src/components/scorecard.rs` | Dual-span labels; `aria-label` on `cell_view` |
| `src/components/tab_bar.rs` | `aria-current` on active tab |
| `src/components/end_game.rs` | `role="dialog"` + `aria-modal` |
| `src/components/resume.rs` | `role="dialog"` + `aria-modal` |
| `src/components/confirm_zero.rs` | `role="dialog"` + `aria-modal`; drop `hide_tab_bar` explicit sets |
| `src/components/confirm_quit.rs` | `role="dialog"` + `aria-modal` |
| `src/components/game_view.rs` | Use `PendingZero` from context |
| `src/app.rs` | Create + provide `PendingZero`; update Effect |
| `style/main.css` | Label toggle CSS; expand 599px block |
| `src/state/game.rs` | Mini-game test |
| `src/worker/advisor.rs` | DP table sanity test |
| `tests/integration.rs` | 4–5 new browser integration tests |
| `e2e/smoke.spec.ts` | Full game + mobile viewport tests |

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Removing `hide_tab_bar.set(false)` from `confirm_zero.rs` means tab bar visibility is now driven entirely by the app.rs Effect. If another signal in the Effect changes *while* confirm-zero is open, the Effect would re-run but `pending_zero` is still `Some`, so hide stays true. Correct. | No issue — Effect is the single source of truth. |
| Simplicity | Promoting `pending_zero` to context adds a newtype + extra signal. Simpler alternative: add an Effect inside `game_view.rs` that syncs `pending_zero` to `hide_tab_bar`. | Rejected: that creates two competing signals writing to `hide_tab_bar`. Context promotion keeps a single authoritative writer. |
| Coupling | `app.rs` now depends on `PendingZero` being meaningful at all times, including when `GameView` is not mounted. | Fine — `pending_zero` defaults to `None` (never mounted = never showing confirm-zero). |
| Performance | Dual `<span>` elements per row label (13 rows × 2 spans = 26 extra DOM nodes) in both Scorecard and ScorecardReadOnly. | Negligible — static text nodes, no reactive closures. |
| Testability | The `pending_zero` context signal can be tested by mounting the full `App` and triggering a zero-cell click, then asserting tab bar is hidden. | Covered by new integration test. |

---

## Implementation Notes

See milestone doc for full summary. Key points:
- PendingZero context promotion makes app.rs the single source of truth for hide_tab_bar
- All hide_tab_bar.set() calls removed from game_view.rs and confirm_zero.rs
- Dual-span label CSS toggle is pure CSS — no JS/signal needed
- Cell aria-labels computed inside existing reactive `move ||` closure in cell_view
- Ask Grandma worker round-trip waived (test origin limitation)

---

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `ROW_LABELS_SHORT` has 13 entries, matches `ROW_COUNT` | 1 | ✅ | compile-time const assert |
| Dual-span labels render in DOM | 3 | ✅ | integration.rs |
| `aria-current="page"` on active tab | 3 | ✅ | integration.rs |
| Overlay `role="dialog"` present | 3 | ✅ | integration.rs |
| `hide_tab_bar` true when `pending_zero` Some | 3 | ✅ | integration.rs |
| Mini-game: 78 cells → `is_game_complete` + `grand_total` | 1 | ✅ | game.rs #[test] |
| DP table: `V_COL[8191] == 0.0` | 2 | ✅ | advisor.rs #[wasm_bindgen_test] |
| DP table: `V_COL[0] > 200.0` | 2 | ✅ | advisor.rs #[wasm_bindgen_test] |
| Resume round-trip | 1 | ✅ | existing `game_state_json_round_trip` |
| Ask Grandma worker round-trip | — | ❌ waived | Requires serving worker files from test origin; not supported by wasm-pack test runner without custom HTTP fixture setup. Covered by E2E smoke test `grandma panel shows at least one action card`. |
| Full game completion E2E | E2E | ✅ | smoke.spec.ts |
| Mobile viewport no-overflow | E2E | ✅ | smoke.spec.ts |

---

## Test Results

- Native: 94 tests pass (cargo test)
- Browser: 60 tests pass (wasm-pack test --headless --firefox)
- E2E: 2 new M10 tests pass; 4 M8 theme tests have pre-existing failures

---

## Review Notes

Code review by Copilot agent found one issue: redundant `hide_tab_bar.set()` calls remaining in
`game_view.rs` after the PendingZero promotion. Fixed: removed all 3 explicit sets (lines 182, 190, 195).
The app.rs Effect is now the sole writer to hide_tab_bar.

---

## Callouts / Gotchas

- `ConfirmZero` currently calls `hide_tab_bar.set(false)` in its cancel/confirm handlers.
  These calls must be removed when promoting to Effect-managed tab bar, otherwise the Effect
  and component will fight. Test: confirm tab bar correctly re-shows after dismissing
  confirm-zero.
- `game_view.rs` had `HideTabBar` imported — all 3 explicit `hide_tab_bar.set()` calls were
  removed. The import was dropped entirely since the component no longer needs it.
- The `@media (max-width: 599px)` block previously set `.scorecard td:first-child` min-width to
  5.5rem. Updated to `min-width: 2.4rem; max-width: 2.8rem` for abbreviated labels.
- Ask Grandma worker round-trip test was waived — wasm-pack test runner cannot serve worker
  files from test origin without custom HTTP fixture setup.
