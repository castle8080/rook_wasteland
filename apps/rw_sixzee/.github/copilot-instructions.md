# rw_sixzee — Copilot Instructions

A Leptos 0.8 (CSR) + Trunk WASM app. Client-side only; deployed as static files
with no server-side rendering or backend API. Part of the Rook Wasteland monorepo
but developed independently. Deployed under `/rw_sixzee/`.

---

## Project Documentation

All project documentation lives in `doc/`. **Always consult the relevant doc
before implementing a feature.** The docs are the source of truth for design
decisions — do not make architectural choices that contradict them without
updating the doc first.

| File | Purpose | When to read |
|---|---|---|
| `doc/prd.md` | Product Requirements Document — user stories, functional requirements, UI layout, success criteria. | Before implementing any user-facing feature. |
| `doc/tech_spec.md` | Technical Specification — stack, module layout, state architecture, component responsibilities. | Before writing any code. |
| `doc/wireframes.md` | Screen-by-screen UI wireframes — layout, component hierarchy, navigation flows, tab bar visibility rules. | Before implementing any UI component or screen. |
| `doc/project_plan.md` | Milestone overview — milestones in dependency order, status summary. | To understand overall project progress. |
| `doc/milestones/m<N>-*.md` | Per-milestone detail — deliverables, acceptance criteria, task checklist, implementation notes. Files: `m1-bootstrap.md` through `m10-polish-mobile.md`. | Before starting any task within a milestone; after completing a task — update the checklist and add implementation notes. |
| `doc/lessons.md` | Living record of non-obvious bugs, browser/crate quirks, and hard-won insights discovered during development. | **Before starting work in any area.** **After resolving any non-trivial issue** — add a new lesson. |
| `doc/grandma_soul.md` | Grandma's character soul document — her personality, worldview, voice register, vocabulary, what she respects, what she won't say, emotional tier definitions, and a quote generation checklist. | **Before writing or reviewing any Grandma quote or dialogue.** Use as the prompt preamble when generating new content. |

---

## Build, Test, and Lint

Commands run from the **`apps/rw_sixzee/` directory**:

```bash
python make.py build    # debug WASM build (trunk build)
python make.py test     # cargo test (native) + wasm-pack test --headless --firefox
python make.py dist     # release build → dist/
python make.py lint     # cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

Run a single unit test by name:
```bash
cargo test <test_name>
```

Lint with zero-warnings policy:
```bash
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

Required toolchain: `rustup target add wasm32-unknown-unknown`, `cargo install trunk`,
`cargo install wasm-pack` (for browser tests).

`Trunk.toml`: `public_url = "/rw_sixzee/"` — all asset paths (WASM, JS glue, CSS)
are injected as absolute paths under this prefix.

---

## Architecture

```
src/
├── lib.rs              # WASM entry point; #[wasm_bindgen(start)]; lint attrs
├── app.rs              # Root App component; provides app state via context
├── routing.rs          # Hash-based Route enum (no leptos_router)
├── state/
│   └── app_state.rs    # AppState — top-level RwSignals
└── components/
    └── ...             # UI components
```

**`Cargo.toml` crate type:**
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```
`rlib` enables `cargo test` on the native host for pure-logic unit tests.
`cdylib` is the WASM output.

---

## State Architecture

App state is a struct of `RwSignal`s, created in `App` and provided via Leptos
context. All UI controls read/write signals directly.

Any `!Send` web-sys type (e.g. `WebGl2RenderingContext`, `HtmlCanvasElement`) must
be stored as `Rc<RefCell<...>>` — **never inside a Leptos signal**.

---

## Rust / Error Handling

- **Never `.unwrap()`** in non-test code. Use `.expect("why this cannot fail")`.
  The reason must explain *why* the failure is impossible.
- Inside `spawn_local` callbacks where errors cannot be propagated, log to console:
  ```rust
  web_sys::console::error_1(&format!("Failed: {:?}", e).into())
  ```
- Enable in `lib.rs`:
  ```rust
  #![warn(clippy::unwrap_used)]
  #![warn(clippy::todo)]
  ```
- No `unsafe`. No `todo!()` or `unimplemented!()` in committed code.

---

## Leptos 0.8 Patterns

### Signals

```rust
let value = RwSignal::new(0);        // Send + Sync types
let local = RwSignal::new_local(x);  // !Send web-sys types only
```

### Components

```rust
#[component]
fn MyComponent(value: i32, #[prop(optional)] label: Option<String>) -> impl IntoView {
    view! { <div>{value}</div> }
}
```

### on:input

```rust
// Uses web_sys::Event (not InputEvent)
on:input=move |ev| {
    let val = ev.target().unchecked_into::<HtmlInputElement>().value();
}
```

### Context

```rust
provide_context(my_signal);                        // ancestor
let s = expect_context::<RwSignal<MyType>>();      // descendant
```

### Async

```rust
use leptos::task::spawn_local;
spawn_local(async move { ... });   // WASM is single-threaded; no tokio::spawn
```

---

## Routing

**Hand-coded hash routing only — do not use `leptos_router`.**
Hash routing (`#/route`) keeps navigation in the URL fragment, which is never
sent to the server. Required for static file hosting.

App-lifetime `gloo_events::EventListener` (e.g. for `hashchange`) must be kept
alive with `std::mem::forget(listener)`.

---

## WASM Entry Point

```rust
#[cfg(not(test))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

Without `#[wasm_bindgen(start)]` the module starts silently — no error, no app.

---

## web-sys API Notes

- `on:input` → `web_sys::Event`, not `InputEvent`. Read with
  `.target().unchecked_into::<HtmlInputElement>().value()`.
- Canvas 2D: `ctx.set_fill_style_str("colour")` — not the deprecated
  `set_fill_style(&JsValue)`.
- `js_sys::Math::random()` returns `f64` in `[0, 1)` — use for all
  randomization (no `std::time`, not available in WASM).
- For dates in filenames: `js_sys::Date::new_0()` — not `std::time`.
- `gloo_events::EventListenerOptions` defaults to `passive: true`. Any listener
  calling `prevent_default()` must use `EventListenerOptions::enable_prevent_default()`.

---

## Clippy

- `#[allow(clippy::...)]` always requires an explanatory comment.
- State structs with `fn new()` must also `impl Default` delegating to `Self::new()`.
- Run clippy against `wasm32` target — the native and WASM targets can diverge.
- Use `.inspect_err(|e| log(e))` instead of `.map_err(|e| { log(e); e })` for
  side-effect-only error logging (satisfies `clippy::manual_inspect`).

---

## Testing

The project uses three tiers. Every non-trivial task must include tests from the
appropriate tier(s).

- **Tier 1 — native `#[test]`:** pure math, routing logic, string validation. No
  browser needed. Run with `cargo test`.
- **Tier 2 — `#[wasm_bindgen_test]` (low-level):** isolated web-sys API calls.
  Files in `tests/`. Run with `wasm-pack test --headless --firefox`.
- **Tier 3 — `#[wasm_bindgen_test]` (integration):** full component trees mounted
  in headless Firefox; tests signal → DOM reactive wiring.

Each file under `tests/` needs its own `wasm_bindgen_test_configure!(run_in_browser);`.

`.unwrap()` is fine inside any `#[test]` function.

---

## Task Workflow

Before writing code for any non-trivial task:

1. Check `doc/lessons.md` for relevant lessons.
2. Read the relevant milestone doc and sections of `doc/tech_spec.md`.
3. Write a design sketch (data flow, function signatures, edge cases).
4. Implement + write tests (see Testing section above for tier guidance).
5. **Coverage audit:** for every public function or component added or modified,
   list each meaningful behaviour and confirm it is tested or explicitly waived.
6. Run `python make.py lint` and `python make.py test` — both must pass.
7. Every public `fn`/`struct`/`trait` needs a `///` doc comment; magic numbers need named constants.
8. Stage changes and run the `code-review` agent — fix every flagged bug/logic error.
9. Mark the task complete in the milestone doc.
10. Summarize highlights in the milestone doc about implementation.
11. Consider if there were any interesting new lessons during implementation and if important add to doc/lessons.md.
12. Commit.
13. Suggest a set of basic smoke tests to run to test the milestone.

### Commit message format

```
M1.2: scaffold Leptos app entry point

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Format: `M<milestone>.<task>: <imperative description>`. Co-authored-by trailer on every commit.
