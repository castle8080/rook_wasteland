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

## L11: `main` symbol missing in wasm-pack test — duplicate start symbols

**Milestone:** M4  
**Area:** Build / Testing  
**Symptom:** `wasm-pack test --headless --firefox` fails with:
`Error: main symbol is missing, may be because there are multiple exports with the
same name but different signatures, and discarded by wasm-ld`.
Affects every test binary that imports from `rw_teleidoscope`.
`tests/scaffold.rs` (which does NOT import the library) passes fine.  
**Cause:** `src/lib.rs` has `#[wasm_bindgen(start)] fn main()` guarded by
`#[cfg(all(target_arch = "wasm32", not(test)))]`.  When an integration test binary
(e.g. `tests/integration.rs`) imports the library, the library is compiled as a
*dependency*, and `cfg(test)` is `false` in the library — only `true` in the
integration test crate itself.  So the library's `main` export IS included.
The wasm-bindgen-test harness also generates a `main` export.
wasm-ld sees two `main` symbols and discards both → no `main` → runner fails.  
**Fix / Workaround:** Add a `wasm-test = []` feature to `Cargo.toml`.  Gate the
`#[wasm_bindgen(start)]` function (and its imports) with
`not(feature = "wasm-test")` in addition to `not(test)`.  Update `make.py` to
pass `-- --features wasm-test` when invoking `wasm-pack test`, so the library's
start function is compiled out when building browser test binaries.  
**Watch out for:** `cfg(test)` in a library is only `true` when testing that
specific library (`cargo test --lib`), NOT when the library is compiled as a
dependency of an integration test.  Use a Cargo feature — not `cfg(test)` — to
control symbols that must be excluded from integration test binaries.

---

## L12: `attr:name="value"` syntax fails in Leptos 0.8 `view!` macro

**Milestone:** M7  
**Area:** Leptos / Build  
**Symptom:** Compiler error `expected one of ( ) , . :: ? or an operator, found :`
when using `attr:playsinline="true"` inside the `view!` macro.  
**Cause:** The Leptos 0.8 `view!` macro parser does not support the `attr:` prefix
syntax for setting arbitrary HTML attributes.  The `:` after `attr` is not valid in
that position in the RSX parser.  
**Fix / Workaround:** Use the attribute name directly as a boolean or string
prop: `playsinline=true`, `muted=true`.  Leptos 0.8 accepts all standard HTML
attribute names this way without needing a prefix.  
**Watch out for:** Any documentation or example showing `attr:` prefix syntax —
it may refer to an older Leptos version.  Always test unfamiliar view! attribute
syntax with a quick `cargo clippy --target wasm32-unknown-unknown` before writing
more code.

---

## L13: Check `camera_open` after `getUserMedia` resolves to prevent stream leaks

**Milestone:** M7  
**Area:** Camera / Async  
**Symptom:** Opening the camera overlay then immediately clicking "Cancel" can
leave an unreleased `MediaStream` holding the camera hardware open.  
**Cause:** `getUserMedia` is async — it resolves after the browser permission
prompt, which can take seconds.  If the user dismisses the overlay during that
time, `release_camera()` runs (clearing `CAMERA_STREAM`) before the async task
completes.  The task then calls `store_stream(new_stream)`, overwriting `None`
with a live stream that has no path to release.  
**Fix / Workaround:** After `await`-ing `request_camera()`, check
`camera_open.get_untracked()` before calling `store_stream`.  If `false`, stop
the stream's tracks immediately without storing:
```rust
if !app_state.camera_open.get_untracked() {
    // User closed overlay while prompt was showing — discard stream.
    for i in 0..stream.get_tracks().length() { ... }
    return;
}
camera::store_stream(stream.clone());
```  
**Watch out for:** Any future async camera or media operation where the user
might dismiss the UI before the operation completes.  The general pattern:
always re-validate app state after every `await` that involves user interaction.

---
