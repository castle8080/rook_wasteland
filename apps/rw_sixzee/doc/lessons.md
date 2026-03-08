# rw_sixzee — Lessons Learned

## Purpose

This document is a living record of non-obvious problems, surprises, and hard-won
insights discovered during the build-out of rw_sixzee. It is **not** a task
list or a design doc — it is a memory aid.

When you hit a bug that took time to diagnose, encounter a browser or crate quirk
that isn't obvious from the docs, or find that an assumption in the tech spec was
wrong, **add a lesson here**. Future development (and future AI-assisted sessions)
should check this file before starting work in a relevant area to avoid repeating
the same mistakes.

### What belongs here

- Leptos 0.8 / web-sys API surprises
- Browser compatibility issues discovered during manual testing
- Crate version incompatibilities or missing feature flags
- Build / Trunk asset pipeline gotchas
- Web Worker wiring issues
- localStorage / persistence edge cases
- Performance findings (what was fast / slow in practice vs theory)
- Any decision reversed from the tech spec, and why

### What does NOT belong here

- Tasks or status tracking → use `doc/milestones/`
- Design decisions → use `doc/tech_spec.md` or `doc/prd.md`
- General Rust/WASM knowledge that applies everywhere → use `.github/skills/`

### Format for each lesson

```
## L<N>: <Short title>

**Milestone:** M<N>
**Area:** <e.g. Leptos / Build / Worker / Storage / Events / Testing>
**Symptom:** What went wrong or what was surprising.
**Cause:** Why it happened.
**Fix / Workaround:** What was done to resolve it.
**Watch out for:** Any follow-on risks or related areas to check.
```

> **Note on pre-populated lessons (L1–L8):** These were discovered during
> development of sibling app `rw_teleidoscope` and are reproduced here because
> they apply directly to any Leptos 0.8 + wasm-pack project in this monorepo.
> They are recorded here preventively — rw_sixzee may never hit them if the fix
> is already applied from the start.

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

## L3: Use `inspect_err` instead of `map_err` for pure side-effect logging

**Milestone:** M1
**Area:** Build
**Symptom:** `cargo clippy -- -D warnings` fails with `manual_inspect` lint
when `map_err(|e| { side_effect(e); e })` is used solely to log the error
without transforming it.
**Cause:** Clippy's `manual_inspect` lint detects `map_err` where the closure
returns the argument unchanged.
**Fix / Workaround:** Replace `.map_err(|e| { log(e); e })` with
`.inspect_err(|e| { log(e); })`.
**Watch out for:** Any `map_err` used purely for logging — always prefer
`inspect_err` for zero-transformation side effects. This applies widely in
the `state/` and `worker/` modules where errors are logged then propagated.

---

## L4: Gate wasm32-only modules in `lib.rs` to fix `cargo test` on native

**Milestone:** M1
**Area:** Build / Testing
**Symptom:** `cargo test` (native target) fails because modules that import
browser-only `web_sys` types or `gloo` APIs cannot compile on the native target.
**Cause:** Browser-only APIs (e.g. `web_sys::Worker`, `web_sys::Storage`) do not
exist in the native Rust target. Any module importing them will fail to compile
without target gating.
**Fix / Workaround:** Declare `components`, `app`, `worker`, and any other
browser-only modules as `#[cfg(target_arch = "wasm32")] mod …` in `lib.rs`.
Native `cargo test` then only compiles `state/`, `scoring/`, and other pure-Rust
modules, and the unit tests run cleanly.
Integration test files that use browser APIs must also start with
`#![cfg(target_arch = "wasm32")]` to be excluded from native compilation.
**Watch out for:** Any new module that imports `web_sys`, `gloo`, or WASM-only
crate APIs — add the `#[cfg(target_arch = "wasm32")]` gate in `lib.rs` immediately.

---

## L5: `wasm-test` feature flag required to avoid duplicate `main` symbol

**Milestone:** M1
**Area:** Build / Testing
**Symptom:** `wasm-pack test --headless --firefox` fails with:
`Error: main symbol is missing` or a duplicate-symbol linker error.
All test binaries that import from `rw_sixzee` are affected.
**Cause:** `src/lib.rs` has `#[wasm_bindgen(start)] fn main()` guarded by
`#[cfg(not(test))]`. When an integration test binary imports the library,
the library is compiled as a *dependency*, so `cfg(test)` is `false` inside
the library — the start function IS included. The wasm-bindgen-test harness
also generates a `main` export. wasm-ld sees two `main` symbols and the link
fails.
**Fix / Workaround:** The `wasm-test = []` feature is declared in `Cargo.toml`
and the `#[wasm_bindgen(start)]` function is gated with
`#[cfg(not(feature = "wasm-test"))]`. `make.py test` passes
`-- --features wasm-test` to `wasm-pack test`. The tech spec (§13) already
documents this setup — follow it from day one.
**Watch out for:** `cfg(test)` in a library is only `true` when testing that
specific library (`cargo test --lib`), NOT when the library is compiled as a
dependency of an integration test. Use a Cargo feature — not `cfg(test)` — to
control start symbols in browser test builds.

---

## L6: Trunk `copy-dir` strips the parent path — only the terminal directory name is kept

**Milestone:** M3 / M7
**Area:** Build / Asset pipeline
**Symptom:** Fetch of `grandma_quotes.json` (or any other copied asset) fails
at runtime with a 404 or returns an HTML page body. `resp.ok()` passes but the
content is wrong.
**Cause:** `<link data-trunk rel="copy-dir" href="./assets"/>` copies only the
terminal directory component (`assets`) into `dist/`. The serve path becomes
`/rw_sixzee/assets/grandma_quotes.json` — but if the `href` was
`./assets/quotes`, the serve path would be `/rw_sixzee/quotes/`, not
`/rw_sixzee/assets/quotes/`. Always inspect the actual `dist/` layout after
`trunk build` before hardcoding fetch URLs.
**Fix / Workaround:** Run `trunk build` and check `dist/` to confirm the exact
paths. Use those real paths in fetch calls and `Trunk.toml`. Do not assume the
full `href` path is reproduced in `dist/`.
**Watch out for:** Any future `copy-dir` or `copy-file` asset (JSON pools,
binary DP table, fonts) — verify `dist/` layout before wiring up the fetch URL.

---

## L7: Leptos 0.8 browser integration test patterns

**Milestone:** M5
**Area:** Testing
**Several non-obvious gotchas when writing `tests/integration.rs`:**

1. **`mount_to` import path.** The function lives at `leptos::mount::mount_to`,
   not at the crate root. `leptos::mount_to` does not exist and causes a compile
   error. Use `use leptos::mount::mount_to;`.

2. **DOM isolation.** All tests run in the same browser page and share
   `document.body`. Always append a fresh `<div>` container per test and scope
   DOM queries to that container (`container.query_selector(...)`) rather than
   the whole document. This prevents state leakage between tests.

3. **Flushing Leptos effects.** After mounting or mutating a signal, effects are
   scheduled as microtasks — they have not run yet. Yield with:
   ```rust
   wasm_bindgen_futures::JsFuture::from(
       js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL)
   ).await.unwrap();
   ```
   One yield is usually enough; reactive effects in Leptos 0.8 CSR run in the
   microtask checkpoint.

4. **Creating signals outside a component.** `RwSignal::new(value)` in Leptos 0.8
   uses `Arc`-backed storage and can be created before `mount_to` — no reactive
   owner is required. The signal remains valid for the lifetime of the test
   function, making it possible to mutate signals from outside the component tree.

5. **`query_selector_all` on `Element` requires the `"NodeList"` web-sys
   feature.** Add `"NodeList"` to the web-sys `features` list in `Cargo.toml` or
   the method will not be found even though `"Element"` is enabled.

---

## L8: `attr:name="value"` syntax fails in Leptos 0.8 `view!` macro

**Milestone:** M5
**Area:** Leptos / Build
**Symptom:** Compiler error `expected one of ( ) , . :: ? or an operator, found :`
when using `attr:disabled="true"` or similar inside the `view!` macro.
**Cause:** The Leptos 0.8 `view!` macro parser does not support the `attr:` prefix
syntax for setting arbitrary HTML attributes. The `:` after `attr` is not valid
in that position in the RSX parser.
**Fix / Workaround:** Use the attribute name directly as a boolean or string
prop: `disabled=true`, `readonly=true`. Leptos 0.8 accepts all standard HTML
attribute names this way without needing a prefix.
**Watch out for:** Any documentation or example showing `attr:` prefix syntax —
it may refer to an older Leptos version. Always test unfamiliar `view!` attribute
syntax with a quick `cargo clippy --target wasm32-unknown-unknown` before writing
more code.

---

## L10: `leptos::on_cleanup` requires `Send + Sync`; use `Closure::forget()` for page-lifetime DOM listeners

**Milestone:** M1
**Area:** Events / Leptos
**Symptom:** Compiler error "`(dyn FnMut(&Event))` cannot be sent between threads safely" when passing
a `gloo_events::EventListener` into `on_cleanup(move || drop(listener))`.
**Cause:** `leptos::prelude::on_cleanup` requires its closure to be `Send + Sync + 'static` to
support multi-threaded runtimes. `gloo_events::EventListener` contains a `Box<dyn FnMut>` which is
`!Send`, so it cannot satisfy this bound.
**Fix / Workaround:** For DOM listeners that must live for the entire page lifetime (e.g. a `hashchange`
listener on `window` in the root `App` component), use a raw `wasm_bindgen::closure::Closure` and call
`closure.forget()` after registering it with `add_event_listener_with_callback`. The intentional leak
is correct because the App is never unmounted in a browser SPA.
```rust
let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |_: web_sys::Event| { ... });
window.add_event_listener_with_callback("hashchange", cb.as_ref().unchecked_ref()).expect("...");
cb.forget(); // intentional: App lives for the entire page lifetime
```
**Watch out for:** Any future root-component event listener (resize, visibilitychange, etc.) — use the
same `Closure::forget()` pattern. If a listener must be removable (e.g. inside a component that can
unmount), wrap it in `SendWrapper` or use a `RwSignal<bool>` flag to gate the handler instead of
removing the listener.


**Milestone:** M5
**Area:** Events
**Symptom:** Browser console fills with "Unable to preventDefault inside passive
event listener invocation". The call to `event.prevent_default()` is silently
ignored.
**Cause:** `gloo_events::EventListenerOptions::default()` sets `passive: true`.
`EventListener::new()` uses the default, so any listener created this way is
passive. Calling `event.prevent_default()` inside a passive listener is
forbidden by the browser spec.
**Fix / Workaround:** Use `EventListenerOptions::enable_prevent_default()` (sets
`passive: false`) with `EventListener::new_with_options` for any listener that
calls `prevent_default()`. Only call `prevent_default()` when actually needed —
gate it behind the relevant condition to avoid spurious calls.
**Watch out for:** Every `EventListener::new(...)` that intends to call
`event.prevent_default()` — always use `new_with_options` with
`EventListenerOptions::enable_prevent_default()`. In rw_sixzee this is most
likely to surface on touch/pointer events for the dice hold/unhold interaction
on mobile.

## L11: `bonus_pool > 0` and `bonus_forfeited = true` are mutually exclusive

**Milestone:** M5
**Area:** Scoring / game state invariants
**Symptom:** Appears to be a scoring bug: `compute_grand_total(&s.cells, s.bonus_pool)` is called
without checking `bonus_forfeited`, suggesting forfeited bonuses are counted.
**Cause:** This is not actually a bug. The invariant holds by construction: `bonus_pool` is only
incremented inside `detect_bonus_sixzee` behind a `if !state.bonus_forfeited` guard. `bonus_forfeited`
is only set to `true` when a Sixzee cell is scored as 0 — which can only happen before all six Sixzee
cells are filled. Bonus turns require all six cells filled, so the forfeiture flag can only be set
*before* any bonus points are ever earned. Therefore `bonus_pool > 0` implies `bonus_forfeited = false`.
**Fix / Workaround:** No fix needed. Document this invariant when reviewing scoring code.
**Watch out for:** Any future code path that could set `bonus_forfeited = true` *after* `bonus_pool`
has been credited (e.g. an undo feature). If undo is ever added, this invariant must be re-evaluated.

## L12: Playwright E2E against `trunk serve` — two setup gotchas

**Milestone:** E2E bootstrap
**Area:** Playwright + Trunk
**Symptom 1:** `trunk serve` exits with code 1: "error taking the canonical path to the watch ignore
path". All ignore paths in `Trunk.toml [watch]` must exist on disk at startup — Trunk resolves them
eagerly. Creating the directories (e.g., `mkdir test-results`) before starting trunk fixes it.
**Symptom 2:** Tests that wait for Leptos-rendered elements (e.g., `.game-header`) sporadically
time out. The WASM binary is fetched via a dynamic import *after* the HTML `load` event, so
`page.goto()` returns before the WASM is downloaded or JIT-compiled. Using
`page.goto(url, { waitUntil: "networkidle", timeout: 45_000 })` waits for the WASM fetch + the
`grandma_quotes.json` fetch to complete before asserting on Leptos-rendered DOM.
**Symptom 3:** Trunk's live-reload WebSocket causes pages to reload mid-test when Playwright writes
`test-results/` files, causing tests to fail intermittently. Fix: add `--no-autoreload` to
`trunk serve` in the Playwright `webServer` config.
**Watch out for:** If `show_opening_quote` is true, `App` returns the `GrandmaQuoteOverlay` early
and `.game-header` is NOT in the DOM. Smoke tests must check for `(.grandma-quote-overlay || .game-header)`
rather than `.game-header` alone.
