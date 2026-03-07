# Task M10-01: Steampunk Polish

**Milestone:** M10 — Steampunk Polish
**Status:** ✅ Done

## Restatement

This task applies the full steampunk visual theme from PRD Section 8 to the UI chrome.
It loads Cinzel and Courier Prime from Google Fonts, styles the controls panel as aged brass
instrument hardware (bevelled buttons, gauge-rail sliders, lever toggle switches, riveted metal
background), adds SVG icons to the four key action buttons, and implements a collapsible panel
controlled by a new `AppState.panel_open` signal. The canvas WebGL output is unaffected; only
the HTML/CSS shell changes. Scope excludes any changes to rendering logic, shader code, or
application state beyond the new `panel_open` signal.

## Design

### Data flow

`panel_open: RwSignal<bool>` (default `true`) is added to `AppState` and provided via context.
`ControlsPanel` reads `panel_open` to apply `class:is-collapsed` on the root `<aside>`. The
panel-toggle button calls `panel_open.update(|v| *v = !*v)`. `App` also reads `panel_open` to
drive a `class:is-panel-collapsed` on `.main-layout` so the canvas-container can expand via
`flex: 1`.

### Function / type signatures

```rust
// state/app_state.rs
pub struct AppState {
    pub image_loaded:  RwSignal<bool>,
    pub camera_open:   RwSignal<bool>,
    pub camera_error:  RwSignal<Option<String>>,
    pub panel_open:    RwSignal<bool>,   // NEW — true = panel visible
}
```

### Edge cases

- `panel_open` starts `true`; persists only for the session (no localStorage).
- Canvas HTML `width`/`height` attributes remain 800×800 at all times; only CSS
  `max-width: 100%` / `height: auto` affects display size.
- Toggle button always visible at 2.5 rem width regardless of panel state (fixed
  width button at top of overflow:hidden panel).

### Integration points

- `src/state/app_state.rs` — add field
- `src/app.rs` — capture AppState, swap component order, add class
- `src/components/controls_panel.rs` — add toggle button, dividers, toggle-switch wrappers
- `src/components/header.rs` — SVG icons
- `src/components/export_menu.rs` — SVG icon on export button
- `style/main.css` — full CSS rewrite
- `index.html` — Google Fonts link
- `tests/integration.rs` — 10 AppState struct updates + new test
- `tests/m8_export.rs` — 5 AppState struct updates

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | panel_open lost on page reload | Acceptable per PRD — no localStorage persistence required |
| Simplicity | Many CSS changes in one task | All CSS lives in a single file; changes are additive |
| Coupling | panel_open in AppState couples layout to business state | Milestone spec mandates AppState per note in m10 doc |
| Performance | `transition: width` repaints layout | 2.5 rem → 260px on a single panel; negligible |
| Testability | CSS animated widths not testable in headless | Test class presence (`is-collapsed`) not visual width |

## Implementation Notes

- `overflow: hidden` + fixed-width child (`panel-content { min-width: var(--panel-width) }`)
  clips content cleanly during CSS width transition.
- `height: auto` + `max-width: 100%` on the canvas element (backed by 800×800 HTML attrs)
  makes the canvas scale down when the container is narrower than 800 px.
- Bootstrap Icons MIT-licensed SVG paths used for all four icons.
- SVG inside Leptos 0.8 `view!` macro: Leptos sets SVG namespace automatically for `<svg>`
  root elements. Child elements like `<path d="..."/>` work without namespace override.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| panel_open default = true | 1 | ✅ | unit test in app_state.rs |
| panel toggles on button click | 3 | ✅ | integration.rs |
| .is-collapsed class present when panel_open=false | 3 | ✅ | integration.rs |
| CSS transition exists | — | ❌ waived | not testable in headless; MT checklist item |
| Slider / button visual appearance | — | ❌ waived | covered by manual test checklist |

## Test Results

- 35/35 native tests pass (`cargo test`)
- 18/18 browser tests pass (`wasm-pack test --headless --firefox`)
- `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings` — zero warnings
- `trunk build` — succeeds

## Review Notes

No issues found during self-review or code-review agent pass.

## Callouts / Gotchas

- `impl Default for AppState` must appear **before** `#[cfg(test)] mod tests` block in `app_state.rs`.
  The `clippy::items_after_test_module` lint (under `-D warnings`) rejects items placed after the
  test module. This was fixed during Phase 6.
- Bootstrap Icons SVG paths (MIT licence) are stored as `const &str` in each component file and
  embedded directly in the `view!` macro as `<path d=CONST/>`. Leptos 0.8 handles the SVG
  namespace automatically for `<svg>` root elements — no `attr:` prefix needed.
- Panel collapse uses `overflow: hidden` + CSS `width` transition on `.controls-panel` (260 px → 2.5 rem).
  The inner `.panel-content` has a fixed `min-width: var(--panel-width)` so content doesn't reflow
  during the transition — it just clips.
