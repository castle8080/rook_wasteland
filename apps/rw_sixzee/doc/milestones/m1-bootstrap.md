# M1 — Project Bootstrap

<!-- MILESTONE: M1 -->
<!-- STATUS: COMPLETE -->

**Status:** ✅ COMPLETE
**Depends on:** *(none — first milestone)*
**Required by:** All subsequent milestones

---

## Overview

Establish the complete crate skeleton, build tooling, error infrastructure, routing, CSS architecture, and app shell.
No game logic is implemented here; the goal is a compiling, deployable WASM app with working navigation, correct Trunk
configuration, and the foundational error-handling plumbing that all other milestones depend on.

---

## Success Criteria

- [x] `python make.py build` produces a valid WASM build with no compiler errors or warnings
- [x] `python make.py lint` passes with zero clippy warnings
- [x] `python make.py test` runs (zero tests is acceptable; native harness must not error)
- [x] App loads at `http://localhost:8080/rw_sixzee/` (or trunk serve equivalent) and renders a placeholder game screen
- [x] Hash-based navigation works: manually changing URL hash to `#/history` and `#/settings` renders placeholder
  screens; `#/` and `#/game` render the placeholder game screen
- [x] Tab bar is visible on game, history, and settings screens; clicking each tab updates the hash and active state
- [x] Tab bar is NOT rendered on the resume-prompt placeholder screen
- [x] An `AppError::Storage` posted via `report_error()` causes the `ErrorBanner` to appear without crashing the app
- [x] A fatal `AppError::Internal` posted via `report_error()` causes the `ErrorOverlay` to appear
- [x] `clippy::unwrap_used = "deny"` is enforced in `Cargo.toml` lints

---

## Tasks

### Crate & Build Setup

- [x] Create `Cargo.toml` with all required dependencies:
  `leptos 0.8`, `wasm-bindgen`, `web-sys`, `serde`/`serde_json`, `serde-wasm-bindgen`,
  `gloo-events`, `gloo-net`, `rand` (with wasm-bindgen feature), `thiserror`, `uuid` (with js feature)
- [x] Set `[lints.clippy] unwrap_used = "deny"` in `Cargo.toml`
- [x] Create `Trunk.toml` with `public_url = "/rw_sixzee/"` and `watch.ignore = ["dist", "doc"]`
- [x] Add `[[copy-dir]] path = "assets"` to `Trunk.toml` so `grandma_quotes.json` and future static assets are served alongside the WASM bundle
- [x] Create `assets/` directory with a placeholder `grandma_quotes.json` stub (empty arrays, `"version": 1`)
- [x] Create `index.html` as the Trunk entry point
- [x] Create `make.py` following the monorepo convention

### Source Structure

- [x] Create `src/lib.rs` with `#[wasm_bindgen(start)]` entry point; gate with `cfg(target_arch = "wasm32")`;
  include `not(feature="wasm-test")` guard on `start`
- [x] Create `src/app.rs` with skeleton `App` component holding `RwSignal<Route>` and `RwSignal<Option<AppError>>`
- [x] Create `src/router.rs` with `Route` enum, `parse_hash()`, and `navigate()` functions;
  register `hashchange` event listener in `App` using wasm-bindgen `Closure` (see implementation notes)
- [x] Create `src/error.rs` with `AppError`, `AppResult`, `ErrorSeverity`, and `report_error()` helper;
  implement `From<serde_json::Error>` and `From<wasm_bindgen::JsValue>` for `AppError`
- [x] Create placeholder module stubs: `src/state/mod.rs`, `src/components/mod.rs`,
  `src/dice_svg/mod.rs`, `src/worker/mod.rs`

### Components (Skeleton)

- [x] Create `src/components/game_view.rs` — placeholder `<div>` with "Game" text
- [x] Create `src/components/history.rs` — placeholder `<div>` with "History" text
- [x] Create `src/components/settings.rs` — placeholder `<div>` with "Settings" text
- [x] Create `src/components/resume.rs` — placeholder prompt overlay (no logic yet)
- [x] Create `src/components/error_banner.rs` — dismissible banner shown when
  `app_error.severity() == ErrorSeverity::Degraded`
- [x] Create `src/components/error_overlay.rs` — blocking overlay shown when
  `app_error.severity() == ErrorSeverity::Fatal`; "Start New Game" button clears error signal

### Tab Bar & Navigation

- [x] Implement persistent tab bar component with three tabs: Game, History, Settings
- [x] Tab bar hides (CSS `display: none`) during resume prompt; visible on all other screens
- [x] Active tab highlighted via `.tab-bar__item--active` class
- [x] Clicking tab updates `window.location.hash` and `RwSignal<Route>`

### CSS Architecture

- [x] Create `style/main.css` with CSS custom property declarations in `:root`
  (`--color-bg`, `--color-surface`, `--color-accent`, `--color-text`, `--color-held-border`,
  `--color-preview`, `--font-body`, `--font-display`)
- [x] Add Nordic Minimal theme override block (`[data-theme="nordic_minimal"]`) as default stand-in
  for remaining themes (stubs with same values as `:root` are fine for now)
- [x] Add BEM skeleton blocks: `.tab-bar`, `.tab-bar__item`, `.tab-bar__item--active`,
  `.overlay`, `.error-banner`, `.error-overlay`
- [x] Set `document.body.dataset.theme = "nordic_minimal"` on app init

### rw_index Registration

- [x] Add `rw_sixzee` entry to `apps/rw_index/apps.json` in the monorepo root
- [x] Verify `apps.json` is valid JSON after the addition
- [ ] Note: change `"status"` to `"live"` once the app is deployed (done as part of final release,
  not during this bootstrap milestone)

### Unit Tests

- [x] Add unit tests for `parse_hash()` covering all valid routes (`""`, `"game"`, `"history"`,
  `"history/abc-123"`, `"settings"`) and an unknown hash falling back to `Route::Game`

---

## Notes & Risks

- **Trunk multi-target for Worker:** The Web Worker WASM binary requires a separate Trunk build target. This is
  **not** set up in M1 — it is deferred to M7. For now, `worker/mod.rs` is a stub that returns
  `AppError::Worker("not implemented")`.
- **wasm-bindgen(start) gating:** The `#[wasm_bindgen(start)]` function must be gated with
  `#[cfg(not(feature = "wasm-test"))]` to avoid duplicate symbol errors when running `wasm-pack test`.
  See repository memory: *"wasm testing — cfg(test) is false in a library when compiled as a dependency"*.
- **Empty stub files:** Use `//!` inner doc comments in empty stub files, not `///`. Outer doc comments
  require an attached item and will cause compile errors in empty files.
