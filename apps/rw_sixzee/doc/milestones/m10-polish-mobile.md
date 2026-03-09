# M10 — Polish & Mobile

<!-- MILESTONE: M10 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** M6 (Persistence), M7 (Ask Grandma), M8 (Themes), M9 (History)
**Required by:** *(final milestone — no successors)*

---

## Overview

Complete the project with responsive mobile layout, touch-target enforcement, overlay polish on small screens, full
error handling coverage, and the complete WASM integration test suite. This milestone has no new features — it ensures
every existing feature works correctly on mobile, handles all edge cases gracefully, and the full test suite passes.

---

## Success Criteria

- [ ] At viewport width ≤ 599px, the scorecard renders without horizontal scrolling; row labels are abbreviated
  (1s–6s, 3K, 4K, FH, SS, LS, 6Z, CH); column headers are C1–C6
- [ ] All interactive targets (dice, cells, buttons, tab items, Ask Grandma cards) are ≥ 44×44 px on mobile
- [ ] Tapping a die on iOS Safari and Android Chrome correctly toggles held state
- [ ] Ask Grandma's Advice panel is full-screen (or near-full-screen) on mobile and dismissible without precise pointer control
- [ ] The Zero-score confirmation and Resume prompt are full-screen on mobile and readable without zoom
- [ ] No hover-only states — all essential UI information is visible without hover
- [ ] Tab bar is hidden during: Ask Grandma's Advice panel, Zero-score confirmation, Resume prompt
- [ ] Tab bar is visible (and functional) during: End-of-game summary overlay
- [ ] `python make.py lint` passes with zero warnings
- [ ] `python make.py test` passes: all native unit tests + all WASM integration tests (headless Firefox)
- [ ] Complete mini-game WASM test: stub 6 turns of play via API, verify `is_game_complete()` false;
  fill all 78 cells programmatically, verify grand total matches expected sum
- [ ] Ask Grandma Worker round-trip WASM test returns 5 valid actions
- [ ] Resume round-trip WASM test: serialise `GameState`, "reload" (deserialise), verify all fields equal

---

## Tasks

### Responsive CSS (≤ 599px)

- [ ] Add `@media (max-width: 599px)` block in `style/main.css`
- [ ] Scorecard: shift to condensed grid — abbreviated row labels, smaller cell padding,
  `font-size` reduced, cells shrink to fit 6 columns on screen width
- [ ] Apply abbreviated labels: 1s, 2s, 3s, 4s, 5s, 6s, Sub, Bon, 3K, 4K, FH, SS, LS, 6Z, CH, Tot
- [ ] Column headers: C1–C6 (abbreviated from full numbers)
- [ ] Dice: reduce die size but maintain ≥ 44×44 px tap area via padding
- [ ] Buttons (Roll, Ask Grandma, Apply): full-width or near-full-width on mobile
- [ ] Bonus pool box and grand total: stacked vertically if needed

### Touch Target Enforcement

- [ ] Audit all interactive elements; add `min-height: 44px; min-width: 44px` (or equivalent padding)
  via CSS for all clickable targets in the mobile breakpoint
- [ ] Verify scorecard cells meet 44×44 minimum — may require reducing column count display or
  increasing row height on very narrow screens
- [ ] Verify dice meet minimum with padding; die SVG can be smaller as long as tap area is correct

### Mobile Overlay Handling

- [ ] Ask Grandma's Advice panel: apply `.overlay--grandma` full-screen treatment at `max-width: 599px`
  (position fixed, full viewport height, scrollable card list)
- [ ] Zero-score confirmation: full-screen modal on mobile (same full-viewport fixed positioning)
- [ ] Resume prompt: already full-screen by design; verify correct rendering on small viewports
- [ ] End-of-game summary: verify readable and actionable without zoom on 375px wide screen

### Tab Bar Visibility Rules (Completeness Pass)

- [ ] Audit all overlay components; confirm tab bar is hidden exactly when:
  Ask Grandma's Advice panel open, Zero-score confirmation open, Resume prompt visible
- [ ] Confirm tab bar remains visible when End-of-game summary is shown
- [ ] Implement via CSS class on a parent container or conditional render signal, not duplicated per component

### No Hover Dependency Audit

- [ ] Remove any `:hover`-only CSS that reveals essential information (score previews, held state, tooltips)
- [ ] Score previews must be visible by default when `rolls_used > 0` (always-on, not hover-gated)
- [ ] Held state indicator must be visible without hover (double-border CSS class applied unconditionally)
- [ ] Ask Grandma button disabled state: tooltip ("Ask Grandma unavailable") should be visible as visible text
  or accessible `aria-label`, not only on hover

### Error Handling Completeness

- [ ] Verify all error propagation boundaries from tech spec §13 are implemented:
  - `load_in_progress()` failure → Degraded banner or Fatal overlay (per error type)
  - `load_history()` failure → empty history + Degraded banner
  - `load_theme()` failure → default theme + Degraded banner
  - `roll()` persist failure → Degraded banner, roll not aborted
  - `place_score()` persist failure → Degraded banner, score not aborted
  - `spawn_grandma()` failure → Ask Grandma disabled, no crash
  - `post_grandma_request()` failure → inline Ask Grandma error, no fatal trigger
- [ ] Verify `ErrorBanner` dismissal clears the signal; does not reappear for the same error instance
- [ ] Verify `ErrorOverlay` "Start New Game" clears localStorage in-progress key (best-effort) and resets state

### Accessibility Baseline

- [ ] Add `aria-label` to dice (e.g. `aria-label="Die 1: 5, held"`)
- [ ] Add `aria-label` to scorecard cells (e.g. `aria-label="Fives, Column 3: 15 points, click to score"`)
- [ ] Add `role="dialog"` and `aria-modal="true"` to overlay components
- [ ] Tab bar items have `aria-current="page"` on the active tab

### Full WASM Integration Test Suite (`tests/integration.rs`)

- [ ] Ensure file has `#![cfg(target_arch = "wasm32")]` and `wasm_bindgen_test_configure!(run_in_browser);`
- [ ] **Mini-game test:** Use `GameState::new()`, call `roll()`, call `place_score()` for each of 78 cells
  using `score_for_row()` results; assert `is_game_complete()` returns true; assert `grand_total` > 0
- [ ] **Zero-score confirmation trigger test:** Confirm that calling `place_score()` on a zero-valued
  Sixzee cell sets `bonus_forfeited = true`
- [ ] **Resume round-trip test:** Serialise `GameState` via `serde_json`; deserialise; assert each field equal
- [ ] **Ask Grandma Worker round-trip test:** Spawn worker; send valid `GrandmaRequest`; await response via
  `JsFuture`; assert `actions.len() == 5`
- [ ] **DP table sanity test:** Assert `V_COL[8191] == 0.0_f32`; assert `V_COL[0] > 200.0_f32`

### Final Build & Lint Pass

- [ ] Run `python make.py lint` — zero warnings
- [ ] Run `python make.py test` — all tests pass (native + WASM)
- [ ] Run `python make.py dist` — release build succeeds; `dist/` contains all required files
  including `grandma_worker.js` and `grandma_worker_bg.wasm`
- [ ] Run `python make.py e2e` — all Playwright smoke tests pass
- [ ] **E2E smoke test** (`e2e/smoke.spec.ts`): full end-to-end game completion — drive a game to
  completion by clicking a preview cell each turn (Chance always scores > 0), assert end-game
  overlay appears with a non-zero final score (the single highest-value E2E regression test)
- [ ] **E2E smoke test**: mobile viewport (375 × 812px) — navigate to game, roll, score; assert no
  horizontal overflow (`document.documentElement.scrollWidth <= 375`)
- [ ] Manually verify in Chrome (desktop) and one mobile browser (iOS Safari or Android Chrome):
  - New game flow end-to-end
  - Resume after page reload
  - Ask Grandma's Advice panel with applied recommendation
  - History list and detail view
  - Theme switching
  - Error banner (trigger by disabling localStorage via private browsing)

---

## Notes & Risks

- **WASM test isolation:** Each integration test should use a fresh `GameState` instance and fresh
  DOM container to avoid cross-test contamination. Use a `fresh_container()` helper per the
  repository memory for integration testing patterns.
- **Scorecard cell 44px on mobile:** A 6-column scorecard with 13 rows at 44px minimum height per
  cell would be 572px tall — that's fine vertically. The constraint is horizontal width: 6 columns
  of 44px minimum = 264px, which fits a 375px phone with abbreviated labels and thin borders.
  If cells still overflow, consider a horizontal scroll container for the scorecard as a last resort
  (PRD says "without horizontal scroll" but 6 columns is a hard requirement; this trade-off may need
  revisiting).
- **iOS Safari touch events:** Ensure `gloo_events::EventListener` is used for touch-sensitive
  interactions, not `:hover` CSS. The repository memory notes that `EventListenerOptions` defaults
  to `passive: true`; use `EventListenerOptions::enable_prevent_default()` only if `prevent_default()`
  is needed (unlikely for dice tap).
