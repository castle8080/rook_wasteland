# M1 — Project Bootstrap

<!-- MILESTONE: M1 -->
<!-- STATUS: NOT_STARTED -->

**Status:** 🔲 NOT STARTED
**Depends on:** *(none — first milestone)*
**Required by:** All subsequent milestones

---

## Overview

Establish the complete crate skeleton, build tooling, error infrastructure, routing, CSS architecture, and app shell.
No game logic is implemented here; the goal is a compiling, deployable WASM app with working navigation, correct Trunk
configuration, and the foundational error-handling plumbing that all other milestones depend on.

---

## Success Criteria

- [ ] `python make.py build` produces a valid WASM build with no compiler errors or warnings
- [ ] `python make.py lint` passes with zero clippy warnings
- [ ] `python make.py test` runs (zero tests is acceptable; native harness must not error)
- [ ] App loads at `http://localhost:8080/rw_sixzee/` (or trunk serve equivalent) and renders a placeholder game screen
- [ ] Hash-based navigation works: manually changing URL hash to `#/history` and `#/settings` renders placeholder
  screens; `#/` and `#/game` render the placeholder game screen
- [ ] Tab bar is visible on game, history, and settings screens; clicking each tab updates the hash and active state
- [ ] Tab bar is NOT rendered on the resume-prompt placeholder screen
- [ ] An `AppError::Storage` posted via `report_error()` causes the `ErrorBanner` to appear without crashing the app
- [ ] A fatal `AppError::Internal` posted via `report_error()` causes the `ErrorOverlay` to appear
- [ ] `clippy::unwrap_used = "deny"` is enforced in `Cargo.toml` lints

---

## Tasks

### Crate & Build Setup

- [ ] Create `Cargo.toml` with all required dependencies:
  `leptos 0.8`, `wasm-bindgen`, `web-sys`, `serde`/`serde_json`, `serde-wasm-bindgen`,
  `gloo-events`, `gloo-net`, `rand` (with wasm-bindgen feature), `thiserror`, `uuid` (with js feature)
- [ ] Set `[lints.clippy] unwrap_used = "deny"` in `Cargo.toml`
- [ ] Create `Trunk.toml` with `public_url = "/rw_sixzee/"` and `watch.ignore = ["dist", "doc"]`
- [ ] Add `[[copy-dir]] path = "assets"` to `Trunk.toml` so `grandma_quotes.json` and future static assets are served alongside the WASM bundle
- [ ] Create `assets/` directory with a placeholder `grandma_quotes.json` stub (empty arrays, `"version": 1`)
- [ ] Create `index.html` as the Trunk entry point
- [ ] Create `make.py` following the monorepo convention (`apps/rw_teleidoscope/make.py` is the
  canonical reference): `ROOT = Path(__file__).parent`, `_run(*cmd)` helper with
  `subprocess.run(cmd, cwd=ROOT, check=True)`, one zero-argument function per target dispatched
  via `globals().get(target)`. Targets:
  - `build` → `trunk build`
  - `dist`  → `trunk build --release`
  - `lint`  → `cargo clippy --target wasm32-unknown-unknown -- -D warnings`
  - `test`  → `cargo test` then `wasm-pack test --headless --firefox -- --features wasm-test`
  - `help`  → prints module docstring
- [ ] Verify `python make.py build` compiles cleanly

### Source Structure

- [ ] Create `src/lib.rs` with `#[wasm_bindgen(start)]` entry point; gate with `cfg(target_arch = "wasm32")`;
  include `not(feature="wasm-test")` guard on `start`
- [ ] Create `src/app.rs` with skeleton `App` component holding `RwSignal<Route>` and `RwSignal<Option<AppError>>`
- [ ] Create `src/router.rs` with `Route` enum, `parse_hash()`, and `navigate()` functions;
  register `hashchange` event listener in `App::on_mount`
- [ ] Create `src/error.rs` with `AppError`, `AppResult`, `ErrorSeverity`, and `report_error()` helper;
  implement `From<serde_json::Error>` and `From<web_sys::JsValue>` for `AppError`
- [ ] Create placeholder module stubs: `src/state/mod.rs`, `src/components/mod.rs`,
  `src/dice_svg/mod.rs`, `src/worker/mod.rs`

### Components (Skeleton)

- [ ] Create `src/components/game_view.rs` — placeholder `<div>` with "Game" text
- [ ] Create `src/components/history.rs` — placeholder `<div>` with "History" text
- [ ] Create `src/components/settings.rs` — placeholder `<div>` with "Settings" text
- [ ] Create `src/components/resume.rs` — placeholder prompt overlay (no logic yet)
- [ ] Create `src/components/error_banner.rs` — dismissible banner shown when
  `app_error.severity() == ErrorSeverity::Degraded`
- [ ] Create `src/components/error_overlay.rs` — blocking overlay shown when
  `app_error.severity() == ErrorSeverity::Fatal`; "Start New Game" button clears error signal

### Tab Bar & Navigation

- [ ] Implement persistent tab bar component with three tabs: Game, History, Settings
- [ ] Tab bar hides (CSS `display: none`) during resume prompt; visible on all other screens
- [ ] Active tab highlighted via `.tab-bar__item--active` class
- [ ] Clicking tab updates `window.location.hash` and `RwSignal<Route>`

### CSS Architecture

- [ ] Create `style/main.css` with CSS custom property declarations in `:root`
  (`--color-bg`, `--color-surface`, `--color-accent`, `--color-text`, `--color-held-border`,
  `--color-preview`, `--font-body`, `--font-display`)
- [ ] Add Nordic Minimal theme override block (`[data-theme="nordic_minimal"]`) as default stand-in
  for remaining themes (stubs with same values as `:root` are fine for now)
- [ ] Add BEM skeleton blocks: `.tab-bar`, `.tab-bar__item`, `.tab-bar__item--active`,
  `.overlay`, `.error-banner`, `.error-overlay`
- [ ] Set `document.body.dataset.theme = "nordic_minimal"` on app init

### rw_index Registration

- [ ] Add `rw_sixzee` entry to `apps/rw_index/apps.json` in the monorepo root:
  ```json
  {
    "name":        "Sixzee",
    "slug":        "rw_sixzee",
    "path":        "/rw_sixzee/index.html",
    "icon":        "🎲",
    "tagline":     "Stop trying to force luck on the dice.",
    "description": "Six-column solitaire Sixzee. Seventy-eight cells, each one permanent. The dice don't negotiate. When you're stuck, Ask Grandma — she knows exactly what to do. She always has.",
    "status":      "coming_soon"
  }
  ```
- [ ] Verify `apps.json` is valid JSON after the addition (e.g. `python -c "import json; json.load(open('apps.json'))"`)
- [ ] Note: change `"status"` to `"live"` once the app is deployed (done as part of final release,
  not during this bootstrap milestone)

### Unit Tests

- [ ] Add unit tests for `parse_hash()` covering all valid routes (`""`, `"game"`, `"history"`,
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
