# Bug 002 — Settings aria-label reads signal outside reactive context

## Status
Fixed

## Summary
When the Settings screen mounts, Leptos emits a console warning for every theme
card because the `aria-label` attribute is computed with a plain `format!()`
expression that calls `active_theme.0.get()` outside a reactive tracking context.
The warning fires every time the Settings tab is opened. The visible UI is
currently unaffected, but the `aria-label` does not update reactively when the
active theme changes mid-session, which is a latent accessibility bug.

## Steps to Reproduce
1. Run `trunk serve` (debug build).
2. Open the app in the browser and navigate to the Settings tab.
3. Open the browser DevTools console.
4. **Observed:** Six warnings of the form:
   > `At src\components\settings.rs:41:44, you access a reactive_graph::signal::rw::RwSignal<rw_sixzee::state::theme::Theme> … outside a reactive tracking context.`

## Expected Behavior
No console warnings. The `aria-label` should be wrapped in a reactive closure so
it re-evaluates whenever `active_theme` changes, and Leptos can track the access.

## Actual Behavior
`aria-label` is set to a static `String` computed once at render time.
`active_theme.0.get()` fires outside a tracking context → Leptos warns.
The label value also goes stale if the user switches themes without re-mounting
the Settings component.

## Environment / Context
- **App:** rw_sixzee
- **Build mode:** Debug (trunk serve)
- **Browser:** Firefox (Playwright / manual)
- **Recent changes:** Introduced in M8 `src/components/settings.rs` line 60
- **Error output:**
  ```
  rw_sixzee-9fb07fb216ccc54f.js:813
  At src\components\settings.rs:41:44, you access a
  reactive_graph::signal::rw::RwSignal<rw_sixzee::state::theme::Theme>
  (defined at src\app.rs:91:41) outside a reactive tracking context.
  ```

## Initial Triage
The bug lives in `src/components/settings.rs`, function `theme_card()`, line 60.
The `aria-label` attribute is passed a `String` (result of `format!()`) rather than
a `move || ...` closure, so `is_active()` — which calls `active_theme.0.get()` —
fires once during rendering outside any reactive scope. The fix is a one-line
change: wrap the `format!()` in `move || ...` to make it a reactive closure.
No similar patterns exist in `doc/lessons.md`, but the standard Leptos 0.8 rule
(signal reads inside `view!` must be closures, not eager values) applies directly.

---
<!-- The sections below are filled in during the fix phase -->

## Root Cause
The bug lived in `src/components/settings.rs`, function `theme_card()`, line 60.
The `aria-label` attribute was assigned the result of a plain `format!()` call:
`format!("Select {} theme{}", label, if is_active() { … } else { … })`.
This expression evaluated once during component construction — outside any reactive
tracking scope — so Leptos had no subscriber registered for the `active_theme`
signal read inside `is_active()`. The warning fired on every mount because
`active_theme.0.get()` was called without a tracking context. As a secondary
consequence, the computed string was frozen at render time: switching themes updated
the card's CSS class and checkmark (which were already `move ||` closures) but left
every card's `aria-label` stale. The bug was not caught earlier because no test
verified that `aria-label` updated after a theme switch.

## Fix
Changed `src/components/settings.rs` line 60 from a static `String` to a reactive
closure by prepending `move ||`:

```rust
// Before — reads signal outside reactive context
aria-label=format!("Select {} theme{}", label, if is_active() { " (active)" } else { "" })

// After — tracked closure, re-evaluated on every active_theme change
aria-label=move || format!("Select {} theme{}", label, if is_active() { " (active)" } else { "" })
```

No other files changed in the production fix.

## Regression Test
Added `settings_card_aria_label_updates_on_theme_switch` in `tests/integration.rs`.
The test mounts the full `App` at `/settings`, verifies the default `nordic_minimal`
card's `aria-label` contains `"(active)"`, clicks the Borg card, then asserts that
the Nordic Minimal card's label loses `"(active)"` and the Borg card gains it.
This test would have caught the bug originally: with the static `String`, the label
never changes after the click, so both post-click assertions would have failed.

## Post-Mortem / Lessons Learned

### Leptos `view!` attributes that read signals must always be closures

Any expression passed as an attribute value in the `view!` macro that calls
`.get()` (directly or via a helper) must be wrapped in `move ||`. A plain `format!()`
or conditional expression evaluates once at construction time, outside reactive
tracking. The consequence is two-fold: a runtime console warning (the only visible
symptom), and a stale attribute that never updates.

The compiler cannot catch this. The only signal is the Leptos runtime warning in the
browser console. The affected attribute continues to render its initial value
indefinitely — which is easy to miss in testing if you never exercise the reactive
path after the first render.

Rule: in `view!`, if the expression reads a signal anywhere in its evaluation path,
wrap it in `move || { … }`. This lesson is captured as **L18** in `doc/lessons.md`.

