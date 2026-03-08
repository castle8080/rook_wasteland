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

## L12: Playwright E2E against `trunk serve` — three setup gotchas

**Milestone:** E2E bootstrap
**Area:** Playwright + Trunk
**Symptom 1:** `trunk serve` exits with code 1: "error taking the canonical path to the watch ignore
path". All ignore paths in `Trunk.toml [watch]` must exist on disk at startup — Trunk resolves them
eagerly with `canonicalize()`. Do NOT include ephemeral/generated directories (e.g., `test-results`,
`playwright-report`) in the `[watch] ignore` list — Playwright deletes those directories before
starting the webServer, so they will not exist at trunk startup. Only include directories that are
always present (e.g., `dist`, `doc`, `node_modules`, `e2e`).
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

---

## L13: Leptos context resolves by TypeId — multiple `RwSignal<bool>` silently collide

**Milestone:** M6
**Area:** Leptos / Context
**Symptom:** Child components appear to update the correct overlay signals (e.g.
`show_resume`, `show_opening_quote`) but the UI never responds — the overlay stays
on screen permanently. No compiler error or runtime panic.
**Cause:** `leptos::prelude::provide_context` and `use_context` resolve by
`std::any::TypeId`. When two or more signals of the *same Rust type* are provided,
each `provide_context` call **overwrites** the previous entry. Only the last signal
provided survives in the context map. Every `use_context::<RwSignal<bool>>()` call
in any child therefore returns the same signal (the last one provided), regardless
of the comment describing its purpose.

In rw_sixzee's M6 implementation, `show_resume`, `show_opening_quote`, and
`hide_tab_bar` are all `RwSignal<bool>`. After `provide_context(hide_tab_bar)` ran
last, every `use_context::<RwSignal<bool>>()` returned `hide_tab_bar`. The `ResumePrompt`
component's "Discard and Start New" and "Resume Game" handlers were writing to
`hide_tab_bar` rather than the intended signals, so `show_resume` never became
`false` and the prompt never dismissed.
**Fix / Workaround:** Wrap each `bool` signal in a **unique newtype** so it has a
distinct `TypeId`. Defined in `src/state/mod.rs`:
```rust
#[derive(Clone, Copy)]
pub struct ShowResume(pub RwSignal<bool>);

#[derive(Clone, Copy)]
pub struct ShowOpeningQuote(pub RwSignal<bool>);

#[derive(Clone, Copy)]
pub struct HideTabBar(pub RwSignal<bool>);
```
Provide and look up via the newtype:
```rust
provide_context(ShowResume(show_resume));
// …
let show_resume = use_context::<ShowResume>().expect("…").0;
```
**Watch out for:** This applies to **any** shared Leptos context type — not just
`RwSignal<bool>`. Two `RwSignal<String>`, two `Memo<u32>`, two `ReadSignal<Vec<_>>`
would all silently collide. The pattern is: one distinct Rust type per context slot.
`RwSignal<GameState>` is safe only because there is exactly one in the app.

---

## L14: Setting a signal inside a `view!` reactive closure is an anti-pattern

**Milestone:** M6
**Area:** Leptos / Reactivity
**Symptom:** After dismissing the opening-quote overlay, the tab bar briefly fails
to reappear, or the overlay flickers back on screen before finally hiding. Difficult
to reproduce reliably; depends on reactive evaluation order.
**Cause:** A Leptos reactive closure (the `move || { … }` block inside `view!`) is
a *derived value* — conceptually read-only. Writing to a signal *inside* a reactive
closure triggers a secondary reactive flush: the runtime reads the signals, computes
the view, then processes the signal write, which re-evaluates the closure, which may
write again, and so on. This creates confusing evaluation-order dependencies and can
cause stale or double-renders.

In rw_sixzee M6, `hide_tab_bar.set(true)` was placed inside the `move || {}` view
closure that also renders the overlay. The tab bar hide happened only while the
closure's branch evaluated to the overlay — but on dismissal, the signal write could
race against the next evaluation and leave the tab bar hidden one extra frame.
**Fix / Workaround:** Move any signal write that should track other signals into a
`leptos::Effect`:
```rust
Effect::new(move |_| {
    let quote_visible = show_opening_quote.get() && quote_bank.get().is_some();
    hide_tab_bar.set(quote_visible || show_resume.get());
});
```
The Effect runs after the reactive graph settles, reads its dependencies cleanly,
and writes without causing a re-entry loop (Leptos 0.8 Effects are allowed to write
to signals).
**Watch out for:** Any `signal.set(…)` inside a `move || {}` block that is also
used as a reactive source in `view!`. The rule is: reactive closures *read*; Effects
*write*.

---

## L15: WASM browser tests share localStorage — storage tests must isolate before mounting App

**Milestone:** M6
**Area:** Testing / localStorage
**Symptom:** Integration tests that mount the full `App` pass individually but fail
when run as a suite. Tests that previously showed the game view now show the
`ResumePrompt` overlay instead — causing DOM queries to find 0 dice buttons.
**Cause:** All `#[wasm_bindgen_test]` tests in `tests/integration.rs` run in the
*same headless browser page* and share a single `localStorage`. Storage tests
written for M6 (`save_in_progress`, `save_history`, `save_theme`) leave entries in
`localStorage` under the `rw_sixzee.*` keys. A subsequent test that mounts `App`
triggers the M6 load sequence, which reads those keys, finds a saved game, and shows
the `ResumePrompt` — hiding the game view that the test asserts against.
**Fix / Workaround:** Add a `clear_game_storage()` helper in the `helpers` section
of `tests/integration.rs`:
```rust
fn clear_game_storage() {
    if let Some(storage) = web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
    {
        let _ = storage.remove_item("rw_sixzee.in_progress");
        let _ = storage.remove_item("rw_sixzee.history");
        let _ = storage.remove_item("rw_sixzee.theme");
    }
}
```
Call `clear_game_storage()` at the top of every test that mounts the full `App`.
**Watch out for:** Any future localStorage key added to the `rw_sixzee.*` namespace
must also be removed in `clear_game_storage()`. The same issue affects test
frameworks that run tests in a persistent browser context — always reset shared
browser state at the start of each test that depends on it being clean.

## L16: Web Worker with WASM in a Leptos app — full setup recipe

**Milestone:** M7
**Area:** Web Worker / wasm-bindgen / Trunk

**What you need to do to ship a WASM Web Worker alongside a Leptos CSR app:**

### 1. Cargo feature gate
Add a `worker = []` feature in `Cargo.toml`. Gate the worker entry point on it:
```rust
// src/worker/grandma_worker.rs
#[cfg(all(target_arch = "wasm32", feature = "worker"))]
mod inner {
    #[wasm_bindgen::prelude::wasm_bindgen(start)]
    pub fn worker_start() { ... }
}
```
Gate the main app entry point to exclude it:
```rust
// src/lib.rs
#[cfg(all(target_arch = "wasm32", not(test), not(feature = "wasm-test"), not(feature = "worker")))]
#[wasm_bindgen(start)]
fn main() { ... }
```
This lets the same crate produce two distinct WASM binaries from one `cargo build`.

### 2. Build the worker binary
```bash
cargo build --target wasm32-unknown-unknown --features worker [--release]
wasm-bindgen target/wasm32-unknown-unknown/debug/rw_sixzee.wasm \
  --out-dir dist/assets/ --target no-modules --no-typescript \
  --out-name grandma_worker_core
```
Produces `dist/assets/grandma_worker_core.js` + `dist/assets/grandma_worker_core_bg.wasm`.
Requires `cargo install wasm-bindgen-cli` (version must match `wasm-bindgen` in `Cargo.toml`).

### 3. JS loader
Create `assets/grandma_worker.js` (Trunk copies it to `dist/assets/`):
```js
importScripts('./grandma_worker_core.js');
wasm_bindgen('./grandma_worker_core_bg.wasm').catch(function(e) {
    self.postMessage(String(e));
});
```

### 4. Spawn from the main thread
```rust
let worker = web_sys::Worker::new("./assets/grandma_worker.js")?;
```
The path is relative to the page URL, not the JS file. With `public_url = "/rw_sixzee/"` in
`Trunk.toml` the correct path is `./assets/grandma_worker.js`.

### 5. Trunk hook — so `trunk serve` just works
Without this, the worker binary only gets built when you run `make.py build` explicitly.
Add to `Trunk.toml`:
```toml
[[hooks]]
stage = "post_build"
command = "python"
command_arguments = ["make.py", "worker"]
```
The hook fires after every `trunk build` and every `trunk serve` rebuild, keeping the
worker binary in sync with code changes automatically.

### 6. Suppress lints on generated files
`wasm-bindgen`-generated files (and `include!`-d DP tables) trigger
`clippy::large_const_arrays` and `clippy::excessive_precision`. Wrap the `include!` in
a submodule with inner allows so the suppressions are scoped:
```rust
mod dp_tables {
    #![allow(clippy::large_const_arrays, clippy::excessive_precision)]
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/generated/v_col.rs"));
}
use dp_tables::{V_COL, YZ_BONUS_CORRECTION};
```

### 7. Asset co-location
Trunk's `[[copy-dir]]` places the `assets/` directory itself under `dist/`, so everything
lands at `dist/assets/` — not directly in `dist/`. The `wasm-bindgen --out-dir` path and
the `Worker::new()` URL must both use `./assets/`.

**Watch out for:** The `→` character (U+2192) in Python print strings causes a
`UnicodeEncodeError` when Trunk spawns the hook subprocess on Windows (cp1252 terminal).
Use ASCII `->` in hook script output.
