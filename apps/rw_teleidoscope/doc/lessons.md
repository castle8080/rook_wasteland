# rw_teleidoscope — Lessons Learned

## Purpose

This document is a living record of non-obvious problems, surprises, and hard-won
insights discovered during the build-out of rw_teleidoscope. It is **not** a task
list or a design doc — it is a memory aid.

When you hit a bug that took time to diagnose, encounter a browser or crate quirk
that isn't obvious from the docs, or find that an assumption in the tech spec was
wrong, **add a lesson here**. Future development (and future AI-assisted sessions)
should check this file before starting work in a relevant area to avoid repeating
the same mistakes.

### What belongs here

- WebGL / GLSL gotchas specific to this codebase
- Leptos 0.8 / web-sys API surprises
- Browser compatibility issues discovered during manual testing
- Crate version incompatibilities or missing feature flags
- Performance findings (what was fast / slow in practice vs theory)
- Shader algorithm corrections (e.g. the fold formula needed adjusting)
- Any decision reversed from the tech spec, and why

### What does NOT belong here

- Tasks or status tracking → use `doc/milestones/`
- Design decisions → use `doc/tech_spec.md` or `doc/prd.md`
- General Rust/WASM knowledge that applies everywhere → use `.github/skills/`

### Format for each lesson

```
## L<N>: <Short title>

**Milestone:** M<N>  
**Area:** <e.g. WebGL / Leptos / Shader / Camera / Export / Build>  
**Symptom:** What went wrong or what was surprising.  
**Cause:** Why it happened.  
**Fix / Workaround:** What was done to resolve it.  
**Watch out for:** Any follow-on risks or related areas to check.
```

---

## L1: Module-only stub files require inner doc comments

**Milestone:** M1  
**Area:** Build  
**Symptom:** Compiler error "expected item after doc comment" in stub `.rs` files
that contained only `///` outer doc comments and no Rust items.  
**Cause:** `///` outer doc comments must precede an item (fn, struct, etc.). A
file with only outer doc comments has nothing for them to document.  
**Fix / Workaround:** Use `//!` inner doc comments (module-level) instead of `///`
in stub files that contain no items.  
**Watch out for:** Any future stub module or placeholder file — always use `//!`
until real items are added.

---

## L2: `wasm_bindgen_test_configure!` must be repeated in each integration test file

**Milestone:** M1  
**Area:** Build / Testing  
**Symptom:** `wasm-pack test --headless --firefox` reports "no tests to run" and
prints a message saying the suite is "only configured to run in node.js" — even
though `wasm_bindgen_test_configure!(run_in_browser)` exists in `src/lib.rs`.  
**Cause:** `tests/*.rs` integration tests are compiled as separate crates, so the
configure call in `src/lib.rs` does not apply to them.  
**Fix / Workaround:** Add `wasm_bindgen_test_configure!(run_in_browser);` at the
top of every file under `tests/` that contains `#[wasm_bindgen_test]` tests.  
**Watch out for:** Any new integration test file added to `tests/` — always add
the configure line, otherwise tests silently do nothing in browser mode.

---

## L3: `glow::Context::from_webgl2_context` is not `unsafe` in glow 0.13

**Milestone:** M2  
**Area:** WebGL / Build  
**Symptom:** Compiler warns "unnecessary `unsafe` block" when wrapping the
`from_webgl2_context` call in an `unsafe {}` block.  
**Cause:** In glow 0.13, `Context::from_webgl2_context` is a safe function —
the function signature does not include `unsafe`.  
**Fix / Workaround:** Call it directly without an `unsafe` block.  Remove any
SAFETY doc comment that refers to this call.  
**Watch out for:** Any future upgrade of the `glow` crate may change the
signature; re-check after bumping the version.

---

## L4: Use `inspect_err` instead of `map_err` for pure side-effect logging

**Milestone:** M2  
**Area:** Build  
**Symptom:** `cargo clippy -- -D warnings` fails with `manual_inspect` lint
when `map_err(|e| { side_effect(e); e })` is used solely to log the error
without transforming it.  
**Cause:** Clippy's `manual_inspect` lint detects `map_err` where the closure
returns the argument unchanged.  
**Fix / Workaround:** Replace `.map_err(|e| { log(e); e })` with
`.inspect_err(|e| { log(e); })`.  
**Watch out for:** Any `map_err` used purely for logging — always prefer
`inspect_err` for zero-transformation side effects.

---

## L5: Trunk `copy-dir` strips the parent path — only the terminal directory name is kept

**Milestone:** M2 (bug fix)  
**Area:** Build / Asset pipeline  
**Symptom:** Shader fetch fails at runtime with `ERROR: 0:1: '<' : syntax error`
— the GLSL compiler is receiving an HTML page instead of GLSL source.  
**Cause:** `<link data-trunk rel="copy-dir" href="./assets/shaders"/>` copies
only the terminal directory component (`shaders`) into `dist/`.  The parent
`assets/` prefix is **not** reproduced.  Actual serve path is
`/rw_teleidoscope/shaders/vert.glsl`, not
`/rw_teleidoscope/assets/shaders/vert.glsl`.  
**Fix / Workaround:** Verify actual `dist/` layout after `trunk build` and use
the real path in fetch URLs.  When adding new `copy-dir` assets, inspect
`dist/` first — do not assume the full `href` path is preserved.  
**Watch out for:** Any future `copy-dir` asset (images, fonts, data files) —
check the real `dist/` path before hardcoding fetch URLs.

---

## L6: `ERROR: 0:X: '<' : syntax error` in GLSL means the source is HTML

**Milestone:** M2 (bug fix)  
**Area:** WebGL / Shader  
**Symptom:** `gl.compile_shader` fails with `'<' : syntax error` at line 1.  
**Cause:** The fetched "GLSL source" is actually an HTML document.  SPA
servers often return HTTP 200 with an HTML fallback page for any unmatched
URL, so `resp.ok()` passes but the body is not GLSL.  
**Fix / Workaround:** Check the network tab for the actual response body when
this error appears.  The URL being fetched is wrong — fix the path.  
**Watch out for:** Any new asset fetch URL — always cross-check against the
actual `dist/` layout after a build.

---

## L7: `glow::tex_image_2d` takes `Option<&[u8]>` on WASM, not `PixelUnpackData`

**Milestone:** M3  
**Area:** WebGL / Build  
**Symptom:** Compiler error `expected Option<&[u8]>, found PixelUnpackData<'_>` when
calling `gl.tex_image_2d(...)` with `glow::PixelUnpackData::Slice(Some(&bytes))`.  
**Cause:** `glow 0.13` provides two different signatures for `tex_image_2d` depending
on compile target.  The desktop OpenGL build takes `PixelUnpackData`; the WASM /
WebGL build takes `Option<&[u8]>` directly.  
**Fix / Workaround:** Pass `Some(bytes.as_slice())` as the last argument, not a
`PixelUnpackData` variant.  
**Watch out for:** Any other glow texture functions (e.g. `tex_sub_image_2d`) may
have the same target-specific split — check the actual WASM signature before
assuming the desktop API applies.

---

## L8: Gate wasm32-only modules in `lib.rs` to fix `cargo test` on native

**Milestone:** M3  
**Area:** Build / Testing  
**Symptom:** `cargo test` (native target) fails with `from_webgl2_context not found`
because `renderer/context.rs` calls a WASM-only glow API.  
**Cause:** `glow::Context::from_webgl2_context` is only available when compiling to
`wasm32`; it does not exist in the native OpenGL backend.  
**Fix / Workaround:** Declare `renderer`, `components`, and `app` as
`#[cfg(target_arch = "wasm32")] mod …` in `lib.rs`.  Native `cargo test` then only
compiles `utils`, `state`, and `routing` (pure Rust, no WebGL), and the unit tests
run cleanly.  Integration test files that use WebGL/glow must also start with
`#![cfg(target_arch = "wasm32")]` to be excluded from native compilation.  
**Watch out for:** Any new module that imports `glow` or browser-only `web_sys`
types not available on native — add the cfg gate in `lib.rs` immediately.

---

## L9: Embed shaders with `include_str!()` to enable renderer init in tests

**Milestone:** M3 (refactor)  
**Area:** Build / Testing  
**Symptom:** Attempting to mount the full `App` in a `wasm-pack test` integration
test caused the renderer to fail silently — the WebGL program was never linked and
the canvas stayed blank.  The root cause was that `shader.rs` fetched GLSL source
from `/rw_teleidoscope/shaders/vert.glsl` at runtime.  `wasm-pack test` does not
serve static asset files, so the fetch returned 404.  
**Cause:** `Trunk.toml` copy-dirs the shaders into `dist/` at build time, but
`wasm-pack test` does not invoke Trunk and has no equivalent asset pipeline.  
**Fix / Workaround:** Replaced `fetch_text(url).await` with
`include_str!("../../assets/shaders/vert.glsl")` constants.  This embeds the GLSL
source directly in the WASM binary at compile time, making `create_program()` (and
therefore `Renderer::new()`) fully synchronous and independent of any HTTP server.  
**Watch out for:** Any future shader files added for new effects (M5 etc.) must
also use `include_str!()` — do **not** reintroduce runtime shader fetching.  The
binary size overhead is negligible (shaders are a few hundred bytes each).

---

## L10: Browser integration test patterns for Leptos 0.8

**Milestone:** M3 (refactor)  
**Area:** Testing  
**Several non-obvious gotchas when writing `tests/integration.rs`:**

1. **`mount_to` import path.** The function lives at `leptos::mount::mount_to`,
   not at the crate root. `leptos::mount_to` does not exist and causes a compile
   error. Use `use leptos::mount::mount_to;`.

2. **DOM isolation.** All tests run in the same browser page and share
   `document.body`.  Always append a fresh `<div>` container per test and scope
   DOM queries to that container (`container.query_selector(...)`) rather than the
   whole document.  This prevents state leakage between tests.

3. **Flushing Leptos effects.** After mounting or mutating a signal, effects are
   scheduled as microtasks — they have not run yet.  Yield with:
   ```rust
   wasm_bindgen_futures::JsFuture::from(
       js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL)
   ).await.unwrap();
   ```
   One yield is usually enough; reactive effects in Leptos 0.8 CSR run in the
   microtask checkpoint.

4. **`ImageData::data()` returns `Clamped<Vec<u8>>` under wasm32 clippy.** Use
   `.len()` (Rust slice method), not `.length()` (JS TypedArray method).  Both
   `cargo clippy --target wasm32-unknown-unknown --tests` and the actual runtime
   agree on `.len()`.

5. **`query_selector_all` on `Element` requires the `"NodeList"` web-sys
   feature.** Add `"NodeList"` to the web-sys `features` list in `Cargo.toml` or
   the method will not be found even though `"Element"` is enabled.

6. **Creating signals outside a component.** `RwSignal::new(value)` in Leptos 0.8
   uses `Arc`-backed storage and can be created before `mount_to` — no reactive
   owner is required.  The signal remains valid for the lifetime of the test
   function, making it possible to mutate signals from outside the component tree.

---

## L11: wasm-pack test runner "main symbol missing" with wasm-bindgen 0.2.114

**Milestone:** M4  
**Area:** Build / Testing  
**Symptom:** All `wasm-pack test --headless --firefox` runs fail with
`Error: executing wasm-bindgen over the Wasm file — main symbol is missing, may be because
there are multiple exports with the same name but different signatures, and discarded by
wasm-ld`.  This happens for ALL test targets even with no code changes.  
**Cause:** The wasm-pack-cached `wasm-bindgen-test-runner` binary is
version-mismatched with the project's `wasm-bindgen 0.2.114`.  Each version of
`wasm-bindgen` must be paired with exactly the same version of
`wasm-bindgen-test-runner`; `wasm-pack` caches the runner and does not always
update it correctly.  
**Fix / Workaround:** Delete the cached runner at
`%LOCALAPPDATA%\.wasm-pack\wasm-bindgen-<hash>\` and re-run — `wasm-pack` will
download a compatible version.  Alternatively, install `wasm-bindgen-cli` at
exactly version `0.2.114` with `cargo install wasm-bindgen-cli --version 0.2.114`
and run tests via `cargo test --target wasm32-unknown-unknown` directly.  
**Watch out for:** Any time `wasm-bindgen` is bumped in `Cargo.toml`, the cached
runner must also be updated.  Check `%LOCALAPPDATA%\.wasm-pack\` for stale runners.

---
