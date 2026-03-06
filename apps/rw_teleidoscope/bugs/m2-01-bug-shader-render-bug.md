# Bug: Shader Bug

When loading the app nothing appears on the screen and in the console this is seen:

rw_teleidoscope-8794349b96e97e81.js:336 Shader compile error: ERROR: 0:1: '<' : syntax error

__wbg_error_8d9a8e04cd1d3588	@	rw_teleidoscope-8794349b96e97e81.js:336
$web_sys::features::gen_console::console::error_1::__wbg_error_8d9a8e04cd1d3588::h932213b3b7bb6317 externref shim	@	rw_teleidoscope-8794…e81_bg.wasm:0x9c598
$web_sys::features::gen_console::console::error_1::ha0257ed864812860	@	rw_teleidoscope-8794…e81_bg.wasm:0x87979
$rw_teleidoscope::renderer::shader::create_program::{{closure}}::{{closure}}::h12248c6cefd67e38	@	rw_teleidoscope-8794…e81_bg.wasm:0x8055b
$core::result::Result<T,E>::inspect_err::h58b4d12fa5c1aec9	@	rw_teleidoscope-8794…e81_bg.wasm:0x66f26
$rw_teleidoscope::renderer::shader::create_program::{{closure}}::h256fe0e6f29478ac	@	rw_teleidoscope-8794…7e81_bg.wasm:0x7699
$rw_teleidoscope::renderer::Renderer::new::{{closure}}::hfd80e923d3b4d7d1	@	rw_teleidoscope-8794…7e81_bg.wasm:0xefa3
$rw_teleidoscope::components::canvas_view::__component_canvas_view::{{closure}}::{{closure}}::h804c4250172153ca	@	rw_teleidoscope-8794…e81_bg.wasm:0x157eb
$wasm_bindgen_futures::task::singlethread::Inner::is_ready::hd6a2789c91f987bf	@	rw_teleidoscope-8794…e81_bg.wasm:0x67f66
$wasm_bindgen_futures::task::singlethread::Task::run::{{closure}}::h45890888143a16a7	@	rw_teleidoscope-8794…e81_bg.wasm:0x8e1b3
$wasm_bindgen::convert::closures::_::invoke::{{closure}}::hc4ecc01bad9305ad	@	rw_teleidoscope-8794…e81_bg.wasm:0x8b27b
$core::ops::function::FnOnce::call_once::h6dde6f6562c17653	@	rw_teleidoscope-8794…e81_bg.wasm:0x8aa94
$<core::panic::unwind_safe::AssertUnwindSafe<F> as core::ops::function::FnOnce<()>>::call_once::he6bda1ff931a1779	@	rw_teleidoscope-8794…e81_bg.wasm:0x8b211
$wasm_bindgen::__rt::maybe_catch_unwind::hadcbad84909a9982	@	rw_teleidoscope-8794…e81_bg.wasm:0x8b245
$wasm_bindgen::convert::closures::_::invoke::he82e0736a7dddede	@	rw_teleidoscope-8794…e81_bg.wasm:0x715c9
wasm_bindgen__convert__closures_____invoke__he82e0736a7dddede	@	rw_teleidoscope-8794349b96e97e81.js:665
cb0


## Root Cause

`shader.rs` fetched `/rw_teleidoscope/assets/shaders/vert.glsl` but Trunk's
`copy-dir` with `href="./assets/shaders"` copies only the final directory
component (`shaders`) into `dist/`, **not** the full `assets/shaders/` path.
The actual serve path is `/rw_teleidoscope/shaders/vert.glsl`.

Because the fetch URL didn't match, the server returned an HTML 404/fallback
page with HTTP 200 (SPA fallback). The `resp.ok()` check passed, the HTML was
fed as GLSL source, and the GLSL compiler immediately choked on the leading `<`
character of `<!DOCTYPE html>`.

The error `ERROR: 0:1: '<' : syntax error` is a reliable signal that the shader
source is actually an HTML page — the fetch URL is wrong.

## Fix

Changed fetch URLs in `src/renderer/shader.rs` from:
```
/rw_teleidoscope/assets/shaders/vert.glsl
/rw_teleidoscope/assets/shaders/frag.glsl
```
to:
```
/rw_teleidoscope/shaders/vert.glsl
/rw_teleidoscope/shaders/frag.glsl
```

Also corrected the manual test checklist URL in `doc/milestones/m2-webgl-renderer.md`.

## Test Approach

A meaningful failing-first test for this class of bug (wrong URL string) would
require a live Trunk dev server inside the test harness — not feasible with
`#[wasm_bindgen_test]` in isolation. The correct verification is:

1. Inspect `dist/` after `trunk build` — the actual path is
   `dist/shaders/vert.glsl` (no `assets/` prefix).
2. The manual test checklist item "shader fetch 200 OK in network tab" now
   references the correct URL.

## Lessons

### Trunk `copy-dir` strips the parent path

`<link data-trunk rel="copy-dir" href="./assets/shaders"/>` copies only the
terminal directory component (`shaders`) into `dist/`.  The parent `assets/`
prefix is **not** preserved.  Actual serve path: `/rw_teleidoscope/shaders/`.

### `ERROR: 0:X: '<' : syntax error` means the shader source is HTML

When the GLSL compiler reports a `<` syntax error on the very first line, the
source string almost certainly contains an HTML page (SPA fallback, 404 page,
or redirect) rather than GLSL.  Check the network tab for the actual response
body — the URL is wrong.  This is distinct from a genuine GLSL logic error.

## Instructions

* Assess the bug quickly with potential causes
* Research the code to determine likely cause
* If requried you may ask questions during initial investigation as well
* After the cuase is likely determined, determine if a test can be written first to demonstrate the tests fails
* Fix the bug in code and continue tests which should then pass
* Run a code review of tests
* Update the bug report with root cause and a description of the first
* Consider and write learned lessons in the bug report and if the lessons is generally applicable update the doc/lessons.md

