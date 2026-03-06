# M1 — Project Scaffold

**Status:** ⬜ Pending  
**Depends on:** nothing  
**Unlocks:** [M2 — WebGL Canvas & Basic Renderer](m2-webgl-renderer.md)

---

## Goal

Create the minimum project skeleton so that `python make.py build` succeeds
and the app loads in a browser at `localhost/rw_teleidoscope/` (via `trunk serve`).
No kaleidoscope logic — just a blank page with a title and a canvas placeholder.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Create `Cargo.toml` with all crate dependencies from tech spec Section 2 | ⬜ |
| 2 | Create `Trunk.toml` with `public_url = "/rw_teleidoscope/"` and `copy-dir` for `assets/shaders/` | ⬜ |
| 3 | Create `index.html` with Trunk directives for WASM, CSS, and shader asset copy | ⬜ |
| 4 | Create `make.py` from the template in tech spec Section 12 | ⬜ |
| 5 | Create `src/lib.rs` — `#[wasm_bindgen(start)]`, panic hook init, mount Leptos `App` | ⬜ |
| 6 | Create `src/app.rs` — root `App` component rendering a `<div>` placeholder | ⬜ |
| 7 | Create `src/state/mod.rs`, `params.rs`, `app_state.rs` with all signals stubbed out | ⬜ |
| 8 | Create `src/routing.rs` — `Route` enum with hash-based routing (no `leptos_router`) | ⬜ |
| 9 | Create `style/main.css` with all CSS custom property tokens from tech spec Section 11 | ⬜ |
| 10 | Create `assets/shaders/` directory with placeholder `vert.glsl` and `frag.glsl` | ⬜ |
| 11 | Create empty module stubs: `src/renderer/mod.rs`, `src/camera.rs`, `src/utils.rs` | ⬜ |
| 12 | Verify `python make.py build` exits 0 with no compiler errors | ⬜ |
| 13 | Verify `python make.py lint` exits 0 with zero clippy warnings | ⬜ |

---

## Manual Test Checklist

- [ ] `trunk serve` starts without error
- [ ] Browser opens `http://localhost:8080/rw_teleidoscope/` and shows a page (title visible)
- [ ] No console errors in browser dev tools
- [ ] `python make.py build` exits 0
- [ ] `python make.py lint` exits 0

---

## Notes

- `src/lib.rs` must call `console_error_panic_hook::set_once()` before anything else
  so WASM panics appear as readable messages in the browser console.
- All `src/components/` stubs can return an empty `view! { <></> }` for now.
- The shader `.glsl` files at this stage can contain just a passthrough that outputs
  a solid colour — they only need to be valid GLSL so the asset pipeline works.
