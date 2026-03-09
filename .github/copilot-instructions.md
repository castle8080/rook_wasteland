# Rook Wasteland Monorepo — Copilot Instructions

A collection of independent, client-side WASM web apps deployed as a unified static file hosting solution. All apps are built with Rust, use hash-based routing, and ship with no server-side rendering or backend API requirement.

---

## Monorepo Structure

```
rook_wasteland/
├── apps/
│   ├── rw_chess/         # Chess game vs. 3 AI personalities (Leptos 0.8)
│   ├── rw_defender/      # Arcade vertical shooter (Canvas 2D, pure web-sys)
│   ├── rw_index/         # Landing page / app launcher (static)
│   ├── rw_mixit/         # DJ dual-deck audio mixer (Leptos 0.8, Web Audio API)
│   ├── rw_poetry/        # Poetry reader + voice journal (Leptos 0.8, IndexedDB)
│   ├── rw_teleidoscope/  # Interactive kaleidoscope with WebGL (Leptos 0.8)
│   └── [others]/
├── rw_serve/             # Native Rust HTTP/HTTPS server (Axum + Tokio)
├── doc/                  # Monorepo-level documentation
└── make.py               # [optional] top-level build orchestration
```

Each app under `apps/` is independently developed and builds its own `dist/` directory. All are deployed together under a single domain with `rw_serve` or a static file server; `rw_index` acts as the root landing page.

---

## Build, Test, and Lint

### Running from an app directory

Every app has a **`make.py`** in its root; commands are standardized:

```bash
cd apps/<app_name>
python make.py build    # debug WASM build (or appropriate tool)
python make.py test     # run native tests (+ browser tests if configured)
python make.py dist     # release build → dist/
python make.py lint     # linting for WASM target (if applicable)
```

Each app may support different targets (see specific app instructions).

### Required toolchain

For WASM apps (all Leptos or web-sys apps):
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

For testing WASM with a browser:
```bash
cargo install wasm-pack
```

---

## App Types & Technology Stack

### **WASM CSR Apps (Leptos 0.8)**

- **rw_chess**: Browser-based chess game with three AI personalities
- **rw_mixit**: Dual-deck DJ mixer using Web Audio API
- **rw_poetry**: Poetry reader with voice journaling via IndexedDB
- **rw_teleidoscope**: Interactive kaleidoscope with WebGL effects

**Common traits:**
- Client-side rendering (CSR); no server dependency
- Hash-based routing (`#/route`) for static hosting
- `Cargo.toml` crate type: `["cdylib", "rlib"]` for WASM output + native testing
- `Trunk.toml` with `public_url = "/<app_name>/"` (exact match for subdirectory path)
- `cargo test` for pure-logic unit tests; `wasm-pack test` for browser integration tests

### **WASM Canvas App (Pure web-sys)**

- **rw_defender**: Arcade vertical shooter with raw Canvas 2D rendering

**Traits:**
- No Leptos framework; uses web-sys and `js-sys` directly
- Game loop via recursive `requestAnimationFrame`
- Minimal crate type: `["cdylib"]` (no native testing framework)

### **Static App**

- **rw_index**: HTML landing page / app navigator
- Deployed to the root path; other apps mounted at `/<app_name>/`

### **Native Deployment Server**

- **rw_serve** (binary crate, not WASM)
- Serves combined app distribution as static files with SPA fallback per app subdirectory
- Supports HTTP and HTTPS (pure-Rust TLS via `rustls`)
- Structured request logging (method, path, status, latency, IP, bytes, user-agent)
- Optional; static file servers (GitHub Pages, Netlify, Cloudflare Pages, S3) work equally well

---

## Deployment Model

**No backend required.** All apps are deployed as **static files** (HTML, JS, CSS, WASM binary, assets). Routing is **hash-based** (`#/route`) so URLs never reach the server—unknown paths remain in the browser, enabling fallback to `index.html` for SPA navigation.

**Deployment options:**
- **rw_serve**: Local development or internal-network serving with HTTPS support and structured logging
- **Static file hosts**: GitHub Pages, Netlify, Cloudflare Pages, AWS S3 + CloudFront, nginx, Apache (any static-files-only host works)

All apps' `dist/` directories are combined into a single tree:
```
dist/
├── index.html          # from rw_index (root landing page)
├── [rw_index assets]
├── rw_chess/
│   ├── index.html
│   ├── [assets]
│   └── rw_chess_bg.wasm
├── rw_defender/
│   ├── index.html
│   ├── [assets]
│   └── rw_defender_bg.wasm
├── rw_mixit/
│   ├── index.html
│   ├── [assets]
│   └── rw_mixit_bg.wasm
└── ...
```

Exact `public_url` in `Trunk.toml` is critical—Trunk injects this into all asset paths in `index.html`.

---

## Rust / Error Handling (All Apps)

- **Never `.unwrap()`** in non-test code. Use `.expect("reason why this cannot fail")` for programmer-error invariants. `.unwrap()` is only acceptable inside `#[test]` functions.
- The reason string in `.expect()` must explain *why* the failure is impossible — not just what failed. Example: `"AudioContext.createGain() is infallible per Web Audio spec"` ✓; `"create gain"` ✗.
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

## Leptos 0.8 Patterns (CSR Apps)

For apps using Leptos (rw_chess, rw_mixit, rw_poetry, rw_teleidoscope):

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

### Async (WASM is single-threaded)

Use `spawn_local` — `tokio::spawn` does not apply in WASM:
```rust
use leptos::task::spawn_local;
spawn_local(async move { ... });
```

---

## Hash-Based Routing (All Apps)

Use **hand-coded hash-based routing** — do not use `leptos_router`. Hash routing (`#/route`) keeps navigation in the URL fragment, which is never sent to the server. This is required for static file hosting with no URL-rewrite rules.

### Leptos apps (recommended pattern):

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

### Canvas apps (rw_defender pattern):

Read `window.location.hash()` in the game loop; handle routing separately from game logic.

---

## WASM Module Entry Point

Leptos apps (CSR): `#[wasm_bindgen(start)]` is load-bearing. Without it the module starts silently (no error, no app). Exclude during `wasm-pack test` to avoid duplicate start symbols.

```rust
// lib.rs
#[cfg(not(test))]
#[wasm_bindgen(start)]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#![cfg_attr(any(not(target_arch = "wasm32"), test), allow(dead_code, unused_imports))]
```

Canvas apps: Use `spawn_local` to wrap the game loop initialization so DOM references (NodeRef canvas) are available after the first render.

---

## Canvas / requestAnimationFrame Loop

The recursive `requestAnimationFrame` pattern (for canvas apps like rw_defender):

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

## Clippy Standards (All Apps)

- `#[allow(clippy::...)]` suppressions always require an explanatory comment—never silent suppressions.
- State structs with `fn new()` must also `impl Default` (delegating to `Self::new()`) — clippy `-D warnings` will reject them otherwise.
- `clippy::too_many_arguments` — if suppressed, add a comment; refactor into a struct if count keeps growing.

---

## web-sys API Notes

- `on:input` handlers receive `web_sys::Event`, not `InputEvent`. Read value with `ev.target().unchecked_into::<HtmlInputElement>().value()`.
- Canvas: use `ctx.set_fill_style_str("colour")` / `set_stroke_style_str(...)` — not the deprecated `set_fill_style(&JsValue)`.
- `CanvasRenderingContext2d::save()` / `restore()` return `()` — no `.expect()`. `translate()`, `rotate()`, `arc()` return `Result<(), JsValue>` and do need `.expect()`.
- `AudioBuffer::copy_to_channel` takes `&[f32]` directly (not `&Float32Array`).

---

## Testing Strategy (All WASM Apps)

- **Pure functions** (math, routing logic, state transitions) → `#[cfg(test)]` module in the same file, run with `cargo test`. No browser needed.
- **Browser-dependent code** (Web Audio nodes, canvas, WebGL) → `#[wasm_bindgen_test]` in `tests/`, run with `wasm-pack test --headless --firefox`.
- **Component-level integration** (signal → DOM reactive wiring, multi-component pipelines) → `#[wasm_bindgen_test]` in `tests/integration.rs`, mounting real components in a headless browser.
- Extract math helpers as standalone pure functions specifically so they can be unit-tested natively.
- `.unwrap()` is fine inside `#[test]` functions — a panic is a test failure.
- **Coverage audit:** After implementing, explicitly list every meaningful behaviour (happy path, edge cases, error paths, reactive wiring) and confirm each is tested or document why it is waived. Undocumented gaps are bugs in the test suite.

**Common test setup:**
```bash
cd apps/<app_name>
cargo test                                                    # native unit tests
wasm-pack test --headless --firefox                          # WASM integration tests
```

---

## Task Workflow

Before writing code for any non-trivial task:

1. Create `tasks/<milestone>-<id>-<slug>.md` with a restatement of what's being built, where it lives, and what's out of scope
2. Write a design sketch (data flow, function signatures, edge cases, integration points)
3. Critique the design (correctness, simplicity, coupling, performance, testability)
4. Implement + tests
5. **Coverage audit:** for every public function or component added/modified, list each meaningful behaviour and confirm it is tested or document why it is waived. Undocumented gaps are bugs in the test suite.
6. Run all checks: `cargo test` + `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings` + `trunk build` — all must pass
7. Self-review: every public `fn`/`struct`/`trait` needs a `///` doc comment; magic numbers need named constants; cheddk for logic errors; check for edge cases
8. Commit with proper message format (see below), then mark task doc ✅ Done

### Commit message format

```
T2.3: add kaleidoscope rotation control

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
```

Imperative mood, task ID prefix, Co-authored-by trailer on every commit.

---

## Common File Organization (WASM Apps)

```
apps/<name>/
├── src/
│   ├── lib.rs              # WASM entry point; crate-level lint attrs
│   ├── app.rs or main.rs   # root App component (Leptos) or game loop (canvas)
│   ├── routing.rs          # hash-based routing enum
│   └── [domain logic, components, utilities]
├── tests/
│   └── integration.rs      # #[wasm_bindgen_test] integration tests
├── Cargo.toml              # crate-type = ["cdylib", "rlib"] for WASM + native testing
├── Trunk.toml              # public_url = "/<app_name>/"
├── make.py                 # build/test/dist/lint targets
├── index.html              # Trunk input HTML (mounted by rw_serve)
├── doc/                    # spec, design docs, PRDs, lessons learned
├── tasks/                  # per-task design/status documents
└── dist/                   # build output (git-ignored)
```

---

## Related Documentation

- **Monorepo overview:** See [doc/rook_wasteland_spec.md](doc/rook_wasteland_spec.md) for governance, naming, and overall vision
- **Deployment server:** See [rw_serve/doc/rw_serve_spec.md](rw_serve/doc/rw_serve_spec.md) for HTTP/HTTPS configuration and logging
- **rw_index landing page:** See [apps/rw_index/doc/rw_index_spec.md](apps/rw_index/doc/rw_index_spec.md) for app navigation structure
- **App-specific guidance:** Each app may have a `README.md` or `doc/` folder with additional technical patterns (e.g., Web Audio usage in rw_mixit, WebGL in rw_teleidoscope, Canvas in rw_defender)
