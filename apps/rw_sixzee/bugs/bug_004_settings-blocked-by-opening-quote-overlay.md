# Bug 004 — Settings Route Blocked by Opening-Quote Overlay

## Status
Fixed

## Summary
Navigating to `#/settings` immediately after the app loads shows the Grandma
opening-quote overlay instead of the settings screen. The reactive view closure
in `App` gives the opening-quote overlay unconditional priority over route
rendering — once `show_opening_quote = true` AND the quote bank JSON has loaded
(`bank_ready = true`), the overlay renders regardless of the current route.
Because the E2E tests reach the settings route via hash navigation while the
app is still in this state, all four M8 theme tests find 0 `.settings__theme-card`
elements and time out.

## Steps to Reproduce
1. Open the app at `/rw_sixzee/` in a browser (no saved in-progress game, so
   the opening-quote flow is active).
2. Wait for the WASM + assets (including `grandma_quotes.json`) to fully load —
   the opening-quote overlay becomes visible.
3. Navigate to `#/settings` (e.g. via the tab bar or by editing the URL hash).
4. **Observed:** The Grandma opening-quote overlay remains on screen; the theme
   grid never renders.

Alternatively — run `python make.py e2e`; tests 15–18 (M8 Themes describe block)
all fail with `Received: 0` or a 5 000 ms locator timeout on
`.settings__theme-card`.

## Expected Behavior
Navigating to `#/settings` (or `#/history`) should render the corresponding
screen regardless of whether the opening-quote overlay is pending. The overlay
is a "welcome to your new game" affordance and should not block access to other
top-level routes.

## Actual Behavior
The view closure in `App` short-circuits to the opening-quote overlay when
`show_opening_quote && bank_ready`, skipping the `match route.get()` arm
entirely. The settings (and history) routes are therefore unreachable until the
user dismisses the overlay on the Game screen.

## Environment / Context
- **App:** rw_sixzee
- **Build mode:** debug (trunk serve, E2E via Playwright)
- **Browser:** Chromium (Playwright)
- **Recent changes:** Unknown — may have been introduced with M7 (opening-quote
  overlay added) or M8 (settings + theme picker added); the two features interact
  in a way that was not tested at the time.
- **Error output:**
  ```
  Error: expect(locator).toHaveCount(expected) failed
  Locator:  locator('.settings__theme-card')
  Expected: 6
  Received: 0
  Timeout:  5000ms

  TimeoutError: locator.waitFor: Timeout 5000ms exceeded.
  waiting for locator('.settings__theme-card[data-theme=\'devil_rock\']') to be visible
  ```

## Initial Triage
The bug lives in `src/app.rs` inside the `{move || { ... }}` reactive view closure
(roughly lines 207–230). The priority order is:
1. `if show_opening_quote.get() && bank_ready { return opening_quote_overlay; }`
2. `if show_resume.get() { return resume_prompt; }`
3. `match route.get() { ... }`

Step 1 fires whenever the quote bank is loaded and the user hasn't yet dismissed
the overlay — which is the normal state on first page load. The fix likely needs
to exempt non-Game routes (Settings, History, HistoryDetail) from triggering the
overlay, or to gate the overlay on `route == Route::Game`. No directly related
prior lesson in `doc/lessons.md`, though L12 (Playwright `networkidle` timing)
is relevant — `networkidle` ensures the quote bank JSON is fully fetched, making
`bank_ready = true` and activating the blocker before the test can navigate.

---
<!-- The sections below are filled in during the fix phase -->

## Root Cause
The reactive view closure inside `App` in `src/app.rs` (lines ~207–220) short-circuits to the Grandma opening-quote overlay whenever `show_opening_quote == true && bank_ready == true`, completely bypassing the `match route.get()` arm.  `show_opening_quote` is initialised to `true` on every fresh page load; after Playwright's `networkidle` wait the quote bank JSON has been fetched, so `bank_ready` is also `true` by the time the tests navigate to `#/settings`.  The hash-change updates the `route` signal correctly, but the overlay guard fires first and returns the overlay node — the `SettingsView` is never rendered.  A secondary symptom (caught during the code review) was that the `hide_tab_bar` Effect used the same route-unaware condition, meaning the tab bar would also be hidden on Settings/History while the un-dismissed overlay flags were still set, even though no overlay was visible.  The bug was not caught earlier because the M8 theme tests were written after the M7 opening-quote feature, and no test verified that non-Game routes remained reachable while the overlay state was active.

## Fix
Two files changed.

**`src/router.rs`** — added a new public helper:
```rust
// Before: condition was inlined in app.rs without a route check
show_opening_quote && bank_ready

// After: extracted to router.rs, gated on Route::Game
pub fn opening_quote_visible(show: bool, bank_ready: bool, route: &Route) -> bool {
    show && bank_ready && matches!(route, Route::Game)
}
```
Placing the function in `router.rs` (which has no `#[cfg(target_arch = "wasm32")]` gate) allows it to be covered by Tier-1 `cargo test` without needing a wasm-pack browser test.

**`src/app.rs`** — two call sites updated:

*View closure (primary fix):*
```rust
// Before
if show_opening_quote.get() && bank_ready {

// After
if opening_quote_visible(show_opening_quote.get(), bank_ready, &route.get()) {
```

*`hide_tab_bar` Effect (secondary fix, found in code review):*
```rust
// Before
let quote_visible = show_opening_quote.get() && quote_bank.get().is_some();

// After
let quote_visible = opening_quote_visible(
    show_opening_quote.get(),
    quote_bank.get().is_some(),
    &route.get(),
);
```
This keeps the tab-bar visibility logic consistent with what is actually rendered.

## Regression Test
Six tests added to `src/router.rs` under `#[cfg(test)]`:

| Test name | Scenario |
|---|---|
| `opening_quote_visible_false_on_settings_route` | `show=true, bank_ready=true, route=Settings` → `false` |
| `opening_quote_visible_false_on_history_route` | `show=true, bank_ready=true, route=History` → `false` |
| `opening_quote_visible_false_on_history_detail_route` | `show=true, bank_ready=true, route=HistoryDetail{..}` → `false` |
| `opening_quote_visible_true_on_game_route_when_conditions_met` | `show=true, bank_ready=true, route=Game` → `true` |
| `opening_quote_visible_false_when_show_is_false` | `show=false, bank_ready=true, route=Game` → `false` |
| `opening_quote_visible_false_when_bank_not_ready` | `show=true, bank_ready=false, route=Game` → `false` |

Had these tests existed when the overlay was first wired up in M7, the `Route::Settings` and `Route::History` cases would have been empty/undefined and the gap would have been immediately visible.

## Post-Mortem / Lessons Learned

### Overlay priority logic should always be route-aware

When a reactive view closure has a priority-ordered early-return structure (overlay A > overlay B > normal route content), every guard that returns early must check whether the current route is one where that overlay is semantically appropriate.  An overlay that is conceptually tied to a single screen (the opening-quote overlay belongs to the Game screen) should be gated on the matching route from the start — not patched later.  A useful rule: *for every early-return in a route-switching closure, ask "should this fire regardless of which route the user is on?"*

### Keep derived visibility signals consistent with the view

If an `Effect` computes a derived value (e.g. `hide_tab_bar`) that mirrors what is rendered, its predicate must stay in sync with the view's predicate.  Divergence between the two is hard to spot in code review because the Effect and the view closure are separate reactive subscriptions — neither refers to the other.  Extracting the shared predicate into a named function (`opening_quote_visible`) and calling it from both sites is the reliable way to prevent drift.
