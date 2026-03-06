# rw_teleidoscope — Copilot Instructions

A Leptos 0.8 (CSR) + Trunk WASM app. Client-side only; deployed as static files with no server-side rendering or backend API. Part of the Rook Wasteland monorepo but developed independently.

---

## Build, Test, and Lint

Commands run from the **`apps/rw_teleidoscope/` directory**:

```bash
python make.py build    # debug WASM build (trunk build)
python make.py test     # cargo test (native) [+ wasm-pack test if configured]
python make.py dist     # release build → dist/
python make.py lint     # cargo clippy --target wasm32-unknown-unknown
```

Run a single unit test by name:
```bash
cargo test <test_name>
```

Lint with zero-warnings policy:
```bash
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

Required toolchain: `rustup target add wasm32-unknown-unknown`, `cargo install trunk`.

`Trunk.toml`: the `public_url` must match the deployment subdirectory path exactly (`/rw_teleidoscope/`) so Trunk injects correct absolute paths for the WASM binary, JS glue, and CSS into `index.html`.

---

## Architecture

```
src/
├── lib.rs          # WASM entry point; crate-level lint attrs; #[wasm_bindgen(start)]
├── app.rs          # root App component; provides RwSignal<Route> via context
├── routing.rs      # hand-coded hash routing (Route enum)
├── state/          # Leptos RwSignals for UI-visible state
├── components/     # #[component] functions
└── ...
```

**`Cargo.toml` crate type:**
```toml
[lib]
crate-type = ["cdylib", "rlib"]
```
The `rlib` enables `cargo test` on the native host for pure-logic unit tests. The `cdylib` is the WASM output. Run clippy against the `wasm32` target specifically — the two targets can diverge.

---

## Rust / Error Handling

- **Never `.unwrap()`** in non-test code. Use `.expect("reason why this cannot fail")` for programmer-error invariants. `.unwrap()` is only acceptable inside `#[test]` functions.
- The reason string in `.expect()` must explain *why* the failure is impossible — not just what failed. `"AudioContext.createGain() is infallible per Web Audio spec"` is a reason; `"create gain"` is not.
- When an error cannot be propagated (e.g. inside a `spawn_local` callback), log to the browser console rather than panicking:
  ```rust
  web_sys::console::error_1(&format!("Failed: {:?}", e).into())
  ```
- Enable these lints in `lib.rs` to make every unreviewed `.unwrap()` / `.todo!()` visible:
  ```rust
  #![warn(clippy::unwrap_used)]
  #![warn(clippy::todo)]
  ```
- No `unsafe` blocks. No `todo!()` or `unimplemented!()` in committed code — use a `// TODO:` comment and return an early `Err` or `None` instead.

---

## Leptos 0.8 Patterns

### Signals

```rust
// Read/write pair
let (value, set_value) = signal(0);

// Combined (use when both read and write are needed in the same scope)
let value = RwSignal::new(0);
value.get();
value.set(5);
value.update(|v| *v += 1);

// For !Send types (web-sys objects like AudioBuffer, AudioContext)
let local = RwSignal::new_local(some_web_sys_value);
```

Standard `RwSignal::new()` requires `T: Send + Sync`. Web-sys types are `!Send` — use `RwSignal::new_local()` or `signal_local()` for them.

### Components

```rust
#[component]
fn MyComponent(
    /// The value to display
    value: i32,
    #[prop(optional)] label: Option<String>,
) -> impl IntoView {
    view! { <div>{value}</div> }
}
```

### View macro

```rust
// Static text in quotes; reactive values as signals or closures
view! {
    <div class="container">
        <p>{move || value.get() * 2}</p>
        <p class:active=move || is_active.get()>"Label"</p>
    </div>
}

// Conditional rendering
<Show when=move || condition.get()>
    <Child/>
</Show>

// on:input uses web_sys::Event (not InputEvent)
on:input=move |ev| {
    let val = ev.target().unchecked_into::<HtmlInputElement>().value();
}
```

### Context

```rust
// Provide (in ancestor)
provide_context(my_rw_signal);

// Consume (in any descendant)
let state = expect_context::<RwSignal<MyState>>();
```

### Effects and Memos

```rust
// Recomputes when reactive deps change
let doubled = Memo::new(move |_| count.get() * 2);

// Side effect triggered by signal changes
Effect::new(move |_| {
    let val = count.get();
    // do side effect
});
```

### Async

Use `spawn_local` — WASM is single-threaded, `tokio::spawn` does not apply:
```rust
use leptos::task::spawn_local;
spawn_local(async move { ... });
```

---

## Routing

Use **hand-coded hash-based routing** — do not use `leptos_router`. Hash routing (`#/route`) keeps navigation in the URL fragment, which is never sent to the server. This is required for static file hosting with no URL-rewrite rules.

```rust
// src/routing.rs
#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Main,
    // add routes here
}

impl Route {
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            _ => Route::Main,
        }
    }
    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Main => "#/",
        }
    }
}
```

In the root `App`: read initial hash, create `RwSignal<Route>`, listen for `hashchange`, call `provide_context`. Use `std::mem::forget(listener)` on the `gloo_events::EventListener` — app-lifetime listeners must not be dropped.

Use plain `<a href="#/route">` links; no special component needed.

---

## WASM Entry Point

```rust
// lib.rs — #[wasm_bindgen(start)] is load-bearing.
// Without it the module starts silently (no error, no app).
// Exclude during wasm-pack test to avoid a duplicate start symbol.
#[cfg(not(test))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

// Allow dead-code lints on the native test target
#![cfg_attr(any(not(target_arch = "wasm32"), test), allow(dead_code, unused_imports))]
```

---

## Canvas / rAF Loop

The recursive `requestAnimationFrame` loop pattern:

```rust
// Must be wrapped in spawn_local so NodeRef canvas elements are available after first render
spawn_local(async move {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();
    *g.borrow_mut() = Some(Closure::new(move || {
        // draw pass here
        // use .get_untracked() on signals — .get() creates spurious reactive subscriptions
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));
    request_animation_frame(g.borrow().as_ref().unwrap());
});
```

Two `on:` handlers on the same element cannot share a single Rust closure. Wrap in `Rc<dyn Fn()>` and `.clone()` for each handler.

---

## Clippy

- `#[allow(clippy::...)]` suppressions always require an explanatory comment.
- State structs with `fn new()` must also `impl Default` (delegating to `Self::new()`) — clippy `-D warnings` will reject them otherwise.
- `clippy::too_many_arguments` — if suppressed, add a comment; refactor into a struct if count keeps growing.

---

## web-sys API Notes

- `on:input` handlers receive `web_sys::Event`, not `InputEvent`. Read value with `ev.target().unchecked_into::<HtmlInputElement>().value()`.
- Canvas: use `ctx.set_fill_style_str("colour")` / `set_stroke_style_str(...)` — not the deprecated `set_fill_style(&JsValue)`.
- `CanvasRenderingContext2d::save()` / `restore()` return `()` — no `.expect()`. `translate()`, `rotate()`, `arc()` return `Result<(), JsValue>` and do need `.expect()`.
- `AudioBuffer::copy_to_channel` takes `&[f32]` directly (not `&Float32Array`).

---

## Testing

- **Pure functions** (math, routing logic, state transitions) → `#[cfg(test)]` module in the same file, run with `cargo test`. No browser needed.
- **Browser-dependent code** (Web Audio nodes, canvas, WebGL) → `#[wasm_bindgen_test]` in `tests/`, run with `wasm-pack test --headless --firefox`.
- **Component-level integration** (signal → DOM reactive wiring, multi-component pipelines) → `#[wasm_bindgen_test]` in `tests/integration.rs`, mounting real components in a headless browser.
- Extract math helpers as standalone pure functions specifically so they can be unit-tested natively.
- `.unwrap()` is fine inside `#[test]` functions — a panic is a test failure.
- "At least one test" is a floor. After implementing, explicitly audit coverage: list every meaningful behaviour (happy path, edge cases, error paths, reactive wiring) and confirm each is tested or document why it is waived.

---

## Task Workflow

Before writing code for any non-trivial task:

1. Create `tasks/<milestone>-<id>-<slug>.md` with a restatement of what's being built, where it lives, and what's out of scope
2. Write a design sketch (data flow, function signatures, edge cases, integration points)
3. Critique the design (correctness, simplicity, coupling, performance, testability)
4. Implement + tests
5. **Coverage audit:** for every public function or component added/modified, list each meaningful behaviour and confirm it is tested or document why it is waived. Undocumented gaps are bugs in the test suite.
6. `cargo test` + `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings` + `trunk build` — all must pass
7. Self-review: every public `fn`/`struct`/`trait` needs a `///` doc comment; magic numbers need named constants
8. Stage changes and run the `code-review` agent — fix every flagged bug/logic error; waive findings that don't apply by noting the reason in the task doc
9. Commit, then mark task doc ✅ Done

### Commit message format

```
T2.3: add kaleidoscope rotation control

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Imperative mood, task ID prefix, Co-authored-by trailer on every commit.
