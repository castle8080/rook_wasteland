# Leptos Technical Design Principles & API Practices

> **Version:** Leptos 0.8.x  
> **Build Mode:** CSR (Client-Side Rendering) via Trunk  
> **Target:** wasm32-unknown-unknown

---

## 1. Project Setup

### Toolchain
- Use **Trunk** for CSR builds: `cargo install trunk`
- Ensure WASM target: `rustup target add wasm32-unknown-unknown`
- Add leptos with `csr` feature: `cargo add leptos --features=csr`
- Use `leptos_meta` for `<head>` management
- **Do not use `leptos_router`** — use hand-coded hash routing instead (see §8)

### Cargo.toml Feature Flags
```toml
[dependencies]
leptos = { version = "0.8", features = ["csr"] }
leptos_meta = { version = "0.8" }
# leptos_router is NOT used — see §8 for hash-based routing rationale
```

**Important:** Only ONE of `csr`, `hydrate`, or `ssr` should be active per build target.

### index.html
Trunk needs a minimal `index.html` at the project root:
```html
<!DOCTYPE html>
<html>
  <head>
    <link data-trunk rel="css" href="./style/main.css"/>
  </head>
  <body></body>
</html>
```

### main.rs Entry Point
```rust
use leptos::prelude::*;
use leptos::mount::mount_to_body;

fn main() {
    mount_to_body(App);
}
```

---

## 2. Component Model

### Component Definition
- Components are functions annotated with `#[component]`
- Return type is `impl IntoView`
- Function arguments become props; document them with doc comments

```rust
#[component]
fn MyComponent(
    /// The value to display
    value: i32,
    /// Optional label
    #[prop(optional)]
    label: Option<String>,
) -> impl IntoView {
    view! { <div>{value}</div> }
}
```

### Usage in view! macro
```rust
view! { <MyComponent value=42 label="hello"/> }
```

### Component Props Patterns
- `#[prop(optional)]` — makes prop `Option<T>` defaulting to `None`
- `#[prop(default = expr)]` — sets a default value
- `#[prop(into)]` — auto-converts with `.into()`
- Children passed via `children: Children` prop type

---

## 3. Reactivity System

### Signals (Basic Reactive State)
```rust
// Creates a (getter, setter) pair
let (count, set_count) = signal(0);

// Reading
count.get()          // clones value, tracks reactivity
count.with(|v| ...)  // borrows value by ref
count.read()         // returns read guard (Deref to T)

// Writing
set_count.set(5)             // replace value
*set_count.write() += 1      // mutate in place (returns write guard)
set_count.update(|v| *v += 1) // update via closure
```

### RwSignal (Combined Read/Write Signal)
```rust
let count = RwSignal::new(0);
count.get();          // read
count.set(5);         // write
count.update(|v| *v += 1);
```

### Memos (Derived Reactive Values)
```rust
let doubled = Memo::new(move |_| count.get() * 2);
doubled.get(); // reads derived value, only recomputes when deps change
```

### Effects (Side Effects)
```rust
// Runs when reactive dependencies change
Effect::new(move |_| {
    let val = count.get();
    logging::log!("count changed: {val}");
});
```

### Resources (Async Data)
```rust
let data = Resource::new(
    move || count.get(),           // reactive source (re-fetches when changes)
    |count_val| async move {
        fetch_data(count_val).await
    }
);
// Read in view:
view! {
    <Suspense fallback=|| view! { <p>"Loading..."</p> }>
        {move || data.get().map(|d| view! { <p>{d}</p> })}
    </Suspense>
}
```

### Actions (Async Mutations)
```rust
let save = Action::new(|input: &String| {
    let input = input.clone();
    async move { save_to_server(input).await }
});
save.dispatch("hello".to_string());
```

### Thread Safety Note
- Standard signals require `T: Send + Sync`
- Browser-only `!Send` types (from `web-sys`) need local alternatives:
  - `signal_local()`, `RwSignal::new_local()`, `LocalResource`, `Action::new_local()`

---

## 4. View Macro Syntax

### JSX-Like HTML
```rust
view! {
    <div class="container">
        <h1>"Hello, " {name}</h1>
        <p class:active=is_active>"Status"</p>
    </div>
}
```

### Dynamic Values
- **Static text** must be in quotes: `"Hello"`
- **Reactive values** as signals: `{count}` (tracks changes)
- **One-time values**: `{count.get()}` (renders once, not reactive)
- **Closures** for reactivity: `{move || count.get() * 2}`

### Event Handlers
```rust
on:click=move |_| set_count.update(|n| *n += 1)
on:input:target=move |ev| set_name.set(ev.target().value())
```

### Dynamic Attributes
```rust
// Static attribute
<div class="foo">

// Dynamic attribute
<div class=move || if active { "active" } else { "" }>

// Class toggle
<div class:active=move || is_active.get()>

// Style
<div style="color: red;">
<div style:color=move || if error { "red" } else { "black" }>

// Property (for inputs)
<input prop:value=name />

// Two-way binding (bind:)
<input bind:value=(name, set_name) />
<input type="checkbox" bind:checked=checked />
```

### Control Flow
```rust
// Conditional rendering
view! {
    <Show when=move || count.get() > 0 fallback=|| view! { <p>"Empty"</p> }>
        <p>"Has items"</p>
    </Show>
}

// If in blocks
view! {
    {move || if logged_in.get() {
        view! { <Dashboard/> }.into_any()
    } else {
        view! { <Login/> }.into_any()
    }}
}

// List rendering (keyed)
view! {
    <For
        each=move || items.get()
        key=|item| item.id
        children=move |item| view! { <Item item=item /> }
    />
}
```

### NodeRef (DOM References)
```rust
let el: NodeRef<html::Input> = NodeRef::new();
view! { <input node_ref=el /> }
// Access after mount:
let value = el.get().expect("mounted").value();
```

---

## 5. Component Communication

### Props (Parent → Child)
Pass data and callbacks as props.

### Context (Ancestor → Descendant)
```rust
// Provide (in ancestor)
provide_context(MyState::new());

// Consume (in any descendant)
let state = expect_context::<MyState>();
```

### Callbacks / Closures (Child → Parent)
```rust
#[component]
fn Child(on_click: Callback<MouseEvent>) -> impl IntoView {
    view! { <button on:click=on_click>"Click"</button> }
}

// In parent:
view! { <Child on_click=Callback::new(|_| logging::log!("clicked")) /> }
```

### Signals Passed as Props
Pass `ReadSignal`, `WriteSignal`, or `RwSignal` directly as props.

---

## 6. Styling

### Plain CSS
- Place CSS files at root or in `style/` directory
- Reference in `index.html`: `<link data-trunk rel="css" href="./style/main.css"/>`

### Inline Styles
```rust
<div style="background: #2c2c2c; color: white;">
```

### Dynamic Classes with Tailwind
```rust
// In Cargo.toml add tailwind integration
<div class="grid grid-cols-8 gap-1 bg-gray-800">
```

For chess board: prefer hand-crafted CSS or inline styles for precise control over board squares, pieces, highlights, and animations.

---

## 7. Animation Patterns

### CSS Transitions (Preferred for WASM)
Define CSS classes with transitions, toggle them via signal-driven class bindings:
```rust
<div
    class:highlighted=move || last_move.get() == Some(square)
    class:animate-pulse=move || is_animating.get()
>
```

### JavaScript Interop (web-sys)
For custom animations use `web_sys::Window::set_timeout_with_callback` or `gloo_timers`:
```rust
use gloo_timers::callback::Timeout;
Timeout::new(300, move || set_animating.set(false)).forget();
```

### spawn_local for Async Delays
```rust
use leptos::task::spawn_local;
spawn_local(async move {
    gloo_timers::future::TimeoutFuture::new(300).await;
    set_highlighted.set(None);
});
```

---

## 8. Routing

### Preferred Approach: Hand-Coded Hash-Based Routing

**Do not use `leptos_router` in this project.** Use hand-coded fragment (hash) routing instead.

**Why:**  
`leptos_router` uses the HTML5 History API (`pushState`), which changes the URL path (e.g. `/game`, `/settings`). This requires the server to respond to those paths with `index.html` — a catch-all fallback rule. In environments with limited server configuration (shared static hosts, simple file servers with no rewrite rules), this is not reliably available. Hash-based routing (`/#/game`, `/#/settings`) keeps all navigation entirely in the fragment, which is never sent to the server. The server always serves the same `index.html` regardless of the hash, with zero server configuration required.

This approach is less "modern" than History API routing, but it is universally compatible with static file hosting and is the right trade-off for this deployment target.

### Implementation

```rust
// src/routing.rs

#[derive(Clone, PartialEq, Debug)]
pub enum Route {
    Main,
    Settings,
    About,
}

impl Route {
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/settings" => Route::Settings,
            "#/about"    => Route::About,
            _            => Route::Main,
        }
    }

    pub fn to_hash(&self) -> &'static str {
        match self {
            Route::Main     => "#/",
            Route::Settings => "#/settings",
            Route::About    => "#/about",
        }
    }
}

// Navigate programmatically
pub fn navigate(route: &Route) {
    let _ = web_sys::window()
        .unwrap()
        .location()
        .set_hash(route.to_hash());
}
```

```rust
// In the App root component

let initial_hash = web_sys::window().unwrap().location().hash().unwrap_or_default();
let current_route = RwSignal::new(Route::from_hash(&initial_hash));

// React to browser back/forward navigation
let _listener = gloo_events::EventListener::new(
    &web_sys::window().unwrap(),
    "hashchange",
    move |_| {
        let hash = web_sys::window().unwrap().location().hash().unwrap_or_default();
        current_route.set(Route::from_hash(&hash));
    },
);
_listener.forget(); // app-lifetime listener

provide_context(current_route);

view! {
    <Show when=move || current_route.get() == Route::Main>
        <MainView/>
    </Show>
    <Show when=move || current_route.get() == Route::Settings>
        <SettingsView/>
    </Show>
}
```

### Navigation Links

Use plain `<a>` tags with `href="#/settings"` — no special component needed:

```rust
view! {
    <a href="#/">"Home"</a>
    <a href="#/settings">"Settings"</a>
}
```

Or call `navigate()` imperatively from event handlers:

```rust
view! {
    <button on:click=move |_| navigate(&Route::Settings)>"Settings"</button>
}
```

---

## 9. Error Handling

```rust
// ErrorBoundary catches Result Err values
view! {
    <ErrorBoundary fallback=|errors| view! {
        <p>"Error: " {format!("{:?}", errors.get())}</p>
    }>
        {move || result_signal.get()}
    </ErrorBoundary>
}
```

---

## 10. Performance Best Practices

1. **Pass signals, not values** — avoids re-running component body
2. **Use `.with()` over `.get()`** when cloning would be expensive  
3. **Use `Memo`** for expensive derived computations, not plain closures
4. **`<For>` with `key`** for efficient list diffing — never use index as key if items move
5. **Avoid cloning large structures** — prefer `Arc<T>` inside signals
6. **Keep closures small** — capture only what is needed
7. **`RwSignal`** when you need both read and write in same scope
8. **Don't hold read/write guards** across `.await` or while taking the other guard

---

## 11. Common Patterns for Chess App

### Game State as Context
```rust
// Provided at App root
provide_context(GameState::new());

// Used in board, controls, info panels
let game = expect_context::<GameState>();
```

### Board Square Click Handling
```rust
let on_square_click = move |pos: Position| {
    game.handle_click(pos);
};
view! {
    <div on:click=move |_| on_square_click(square_pos)>
        {piece_view}
    </div>
}
```

### Last Move Highlight
```rust
let last_move = Memo::new(move |_| game.last_move.get());
// In square component:
class:last-move=move || last_move.get().map_or(false, |m| m.contains(pos))
```

### Engine Move Animation
```rust
// After engine computes move, set highlight signal then clear after delay
set_engine_highlight.set(Some(engine_move));
spawn_local(async move {
    gloo_timers::future::TimeoutFuture::new(1500).await;
    set_engine_highlight.set(None);
});
```

---

## 12. Trunk Configuration (Trunk.toml)

```toml
[build]
target = "index.html"
dist = "dist"

[watch]
ignore = ["./target"]

[serve]
port = 8080
open = false
```

---

## 13. Key API Reference (Leptos 0.8)

| Old (< 0.6) | New (0.7+/0.8) |
|---|---|
| `create_signal` | `signal()` |
| `create_memo` | `Memo::new()` |
| `create_effect` | `Effect::new()` |
| `create_resource` | `Resource::new()` |
| `create_action` | `Action::new()` |
| `use_context` | `use_context::<T>()` |
| `provide_context(val)` | `provide_context(val)` (same) |
| `on:input` + `.value()` | `on:input:target` + `.target().value()` |
| `<Route path="/">` | `<Route path=path!("/")>` |

---

## 14. Workspace / Module Organization

For a CSR-only chess app:
```
src/
  main.rs           # mount_to_body(App)
  lib.rs            # pub use components, state, engine
  
  state/
    mod.rs          # re-exports
    game.rs         # GameState, reactive signals
    board.rs        # Board representation (pure data)
    piece.rs        # Piece types, positions
    
  engine/
    mod.rs
    search.rs       # Graph-based minimax/alpha-beta
    eval.rs         # Board evaluation heuristics
    movegen.rs      # Legal move generation
    
  rules/
    mod.rs
    validation.rs   # Move validation
    special_moves.rs # Castling, en passant, promotion
    
  ui/
    mod.rs
    app.rs          # App root component
    board.rs        # BoardView component
    square.rs       # SquareView component
    piece.rs        # PieceView component  
    controls.rs     # Game controls, menus
    info_panel.rs   # Captured pieces, move history
    setup.rs        # Game setup / player name / difficulty
```

---

## 15. Canvas & NodeRef Patterns

### Getting a Canvas Element

Declare a `NodeRef<html::Canvas>` in the component, attach it via `node_ref=`, then access it inside an `Effect` (which runs after DOM mount) or event handler.

```rust
use leptos::html;

let canvas_ref: NodeRef<html::Canvas> = NodeRef::new();

view! {
    <canvas node_ref=canvas_ref width="800" height="200" />
}
```

### Accessing the Canvas After Mount

`Effect::new` runs after the synchronous render pass — i.e., the DOM is available. This is the primary "on mount" hook:

```rust
Effect::new(move |_| {
    let Some(canvas) = canvas_ref.get() else { return };
    // canvas is a leptos HtmlElement<html::Canvas>
    // cast to web_sys::HtmlCanvasElement for the Canvas 2D API:
    use wasm_bindgen::JsCast;
    let canvas: web_sys::HtmlCanvasElement = canvas.into();
    let ctx = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

    draw_initial_frame(&ctx);
});
```

### `NodeRef::on_load` (Alternative)

For one-time initialization that must only run when the element first mounts (not on reactive re-runs):

```rust
canvas_ref.on_load(move |canvas| {
    use wasm_bindgen::JsCast;
    let canvas: web_sys::HtmlCanvasElement = canvas.clone().into();
    // canvas is ready — start rAF loop, initialize WebGL, etc.
    start_raf_loop(canvas);
});
```

### Resizing the Canvas to Its CSS Size

Canvas width/height attributes set the pixel buffer. To match the CSS display size:

```rust
fn fit_canvas_to_display(canvas: &web_sys::HtmlCanvasElement) {
    canvas.set_width(canvas.client_width() as u32);
    canvas.set_height(canvas.client_height() as u32);
}
```

### `dyn_into` / `dyn_ref` for web-sys Type Casts

All web-sys downcasts use `JsCast`:

```rust
use wasm_bindgen::JsCast;

// Fallible (returns Result)
let ctx = js_value.dyn_into::<web_sys::CanvasRenderingContext2d>()?;

// Fallible borrow (returns Option<&T>)
let el = event.target().unwrap().dyn_ref::<web_sys::HtmlInputElement>().unwrap();

// Infallible (panics on wrong type — use only when certain)
let canvas: web_sys::HtmlCanvasElement = leptos_element.unchecked_into();
```

---

## 16. requestAnimationFrame Loop

### The Self-Referential Closure Pattern

`requestAnimationFrame` requires a JS callback that schedules itself recursively. In WASM this requires `Closure` + `Rc<RefCell<Option<Closure>>>`:

```rust
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub fn start_raf_loop<F: Fn() + 'static>(on_frame: F) -> i32 {
    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {
        on_frame();

        // Schedule next frame — reads from `f` which it closes over via `g`
        web_sys::window()
            .unwrap()
            .request_animation_frame(
                f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            )
            .unwrap();
    }));

    web_sys::window()
        .unwrap()
        .request_animation_frame(
            g.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        )
        .unwrap()  // returns the handle (i32) for cancellation
}
```

### Cancelling the Loop

Store the RAF handle (an `i32`) and call `cancel_animation_frame` in `on_cleanup`:

```rust
use leptos::prelude::on_cleanup;

// The handle is just an i32 — Copy, Send, Sync — safe to move into on_cleanup
let handle = start_raf_loop(move || {
    // draw frame...
});

on_cleanup(move || {
    web_sys::window()
        .unwrap()
        .cancel_animation_frame(handle)
        .unwrap();
});
```

### Reading Signals Without Tracking in a rAF Loop

Inside the rAF closure, use `get_untracked()` (or `with_untracked()`) to read signal values without subscribing the closure to the reactive graph. Subscribing inside a rAF loop is harmless but wastes work:

```rust
let is_playing = deck_state.is_playing; // RwSignal<bool>
let current    = deck_state.current_secs; // RwSignal<f64>

start_raf_loop(move || {
    if !is_playing.get_untracked() { return; }

    let pos = current.get_untracked();
    draw_waveform_frame(pos);
});
```

### Writing Low-Frequency Display Values Back to Signals

The rAF loop may write back to signals for values that drive reactive DOM (e.g., VU meter level). This is safe as long as you avoid feedback loops — only write to signals that nothing in the rAF loop reads:

```rust
start_raf_loop(move || {
    let level = sample_analyser_level(&analyser_node);
    vu_level_signal.set(level); // drives a reactive <div> height, not read by rAF
});
```

---

## 17. Bridging JS Promises → Rust Futures

### JsFuture

`wasm_bindgen_futures::JsFuture` wraps a JS `Promise` as a Rust `Future`:

```rust
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;
use js_sys::Promise;

// Any web-sys method returning a Promise can be awaited this way:
let array_buffer: web_sys::ArrayBuffer = JsFuture::from(blob.array_buffer())
    .await
    .expect("array_buffer failed")
    .dyn_into()
    .expect("not an ArrayBuffer");
```

### File Loading: Blob.array_buffer() (preferred over FileReader)

`File` extends `Blob` which has an `array_buffer()` method returning a `Promise<ArrayBuffer>` directly — simpler than the old `FileReader` event callback approach:

```rust
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsCast;

async fn file_to_array_buffer(file: web_sys::File) -> web_sys::ArrayBuffer {
    JsFuture::from(file.array_buffer())
        .await
        .unwrap()
        .dyn_into::<web_sys::ArrayBuffer>()
        .unwrap()
}
```

### Decoding Audio with decodeAudioData

`AudioContext::decode_audio_data` returns a `Promise<AudioBuffer>`:

```rust
async fn decode_audio(
    ctx: &web_sys::AudioContext,
    array_buffer: &web_sys::ArrayBuffer,
) -> web_sys::AudioBuffer {
    JsFuture::from(ctx.decode_audio_data(array_buffer).unwrap())
        .await
        .unwrap()
        .dyn_into::<web_sys::AudioBuffer>()
        .unwrap()
}
```

### spawn_local — Fire-and-Forget Async Tasks

Use `leptos::task::spawn_local` to kick off async work from a synchronous context (e.g., an event handler):

```rust
use leptos::task::spawn_local;

let on_file_selected = move |ev: web_sys::Event| {
    let input = ev.target().unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let file = input.files().unwrap().get(0).unwrap();

    spawn_local(async move {
        let ab  = file_to_array_buffer(file).await;
        let buf = decode_audio(&audio_ctx, &ab).await;
        track_name_signal.set(Some("Loaded".to_string()));
        audio_deck.borrow_mut().buffer = Some(buf);
    });
};
```

`spawn_local` is re-exported from `wasm_bindgen_futures` — either import works:

```rust
use leptos::task::spawn_local;           // preferred in Leptos components
use wasm_bindgen_futures::spawn_local;   // equivalent, works anywhere in WASM
```

---

## 18. Non-Send (web-sys) Types in Leptos

### The Problem

`web-sys` types (`JsValue`, `AudioContext`, `CanvasRenderingContext2d`, etc.) are `!Send`. Leptos 0.8's default `RwSignal<T>` and `Effect::new` require `T: Send + Sync`. This causes compile errors when storing web-sys objects in standard signals.

### Solution: Local Variants

Leptos provides `_local` variants for `!Send` types. Use these for anything that touches web-sys:

```rust
// Instead of:             RwSignal::new(...)
let ctx = RwSignal::new_local(None::<web_sys::AudioContext>);

// Instead of:             signal(...)
let (read, write) = signal_local(None::<web_sys::CanvasRenderingContext2d>);

// Effect::new already uses spawn_local internally — safe for !Send closures
// as long as the captured types are 'static (not borrowed)
Effect::new(move |_| { /* web-sys access is fine here */ });
```

### Rc\<RefCell\<T\>\> for Shared Mutable web-sys Objects

For audio graph structs or other non-Clone web-sys wrappers that need to be shared between closures and the Leptos component, use `Rc<RefCell<T>>`:

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Shared audio deck: referenced by component, rAF loop, and event handlers
let audio_deck: Rc<RefCell<AudioDeck>> = Rc::new(RefCell::new(AudioDeck::new(&ctx)));

// Clone the Rc to share into multiple closures:
let deck_for_raf    = audio_deck.clone();
let deck_for_events = audio_deck.clone();

// Access:
deck_for_raf.borrow().gain.gain().set_value(0.8);
deck_for_events.borrow_mut().buffer = Some(audio_buffer);
```

### StoredValue for Non-Clone !Send Values in Leptos Context

When you need to store a `!Send` value in Leptos context across component boundaries without cloning, use `StoredValue::new_local`:

```rust
use leptos::prelude::StoredValue;

let deck = StoredValue::new_local(AudioDeck::new());
provide_context(deck);

// In child components:
let deck = expect_context::<StoredValue<AudioDeck>>();
deck.with_value(|d| d.gain.gain().set_value(0.5));
```

---

## 19. File Input Handling

### The File Picker Pattern

Use a hidden `<input type="file">` triggered by a styled button. Attach `on:change` to handle the selected file:

```rust
let file_input_ref: NodeRef<html::Input> = NodeRef::new();

let on_load_click = move |_| {
    // Programmatically click the hidden input
    if let Some(input) = file_input_ref.get() {
        input.click();
    }
};

let on_file_change = move |ev: web_sys::Event| {
    let input = ev.target().unwrap()
        .dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let Some(file) = input.files().and_then(|f| f.get(0)) else { return };

    spawn_local(async move {
        let ab  = file_to_array_buffer(file.clone()).await;
        let buf = decode_audio(&ctx, &ab).await;
        track_name.set(Some(file.name()));
        audio_deck.borrow_mut().buffer = Some(buf);
    });
};

view! {
    // Hidden native file input
    <input
        node_ref=file_input_ref
        type="file"
        accept="audio/mpeg,audio/wav,audio/ogg,audio/flac,audio/aac"
        style="display:none"
        on:change=on_file_change
    />
    // Styled trigger button
    <button on:click=on_load_click>"Load Track"</button>
}
```

---

## 20. Cleanup and Lifecycle

### on_cleanup — Dispose Resources When Component Unmounts

`on_cleanup` registers a function that runs when the reactive owner (component scope) is disposed. Use it to cancel rAF loops and drop event listeners:

```rust
use leptos::prelude::on_cleanup;

// rAF handle is i32 — Copy, satisfies Send + Sync on all targets
let raf_handle = start_raf_loop(/* ... */);
on_cleanup(move || {
    web_sys::window().unwrap().cancel_animation_frame(raf_handle).unwrap();
});
```

### gloo-events EventListener Lifetime Management

`gloo_events::EventListener` removes its DOM listener on `Drop`. To keep it alive for the component lifetime, hold it in a signal or leak it deliberately:

```rust
use gloo_events::EventListener;

// Hold for component lifetime — dropped (and listener removed) when component unmounts
let _listener = EventListener::new(&web_sys::window().unwrap(), "keydown", move |e| {
    // handle keydown
});
// Store in a StoredValue or just let it live — if the component never unmounts, .forget() is fine:
_listener.forget(); // converts to a static listener; only do this for app-lifetime listeners
```

### Effect as "On Mount" Hook

Effects run after the first render, so they serve as `onMount` equivalents:

```rust
Effect::new(move |_| {
    // Runs once after DOM paint — NodeRefs are populated by this point
    let canvas = canvas_ref.get().expect("canvas mounted");
    // ... initialize canvas, start rAF loop, etc.
});
```

To run only once and not re-subscribe to reactive dependencies, use `with_untracked` inside the effect or `Effect::watch` with an explicit dependency list.

---

## 21. Unit Testing

### Two Testing Layers

| Layer | Tool | Target | What it tests |
|---|---|---|---|
| **Native unit tests** | `#[test]` + `cargo test` | host (x86/ARM) | Pure Rust logic with no DOM or web API dependency |
| **WASM browser tests** | `#[wasm_bindgen_test]` + `wasm-pack test` | `wasm32-unknown-unknown` | Reactive signals, DOM rendering, Leptos components, web API interactions |

Use native tests as much as possible (they are fast). Reach for WASM browser tests when you need a real DOM, real Web Audio API, or need to verify that a Leptos component renders correctly.

---

### Layer 1: Native `#[test]` (Pure Rust Logic)

Any code that does not touch `web_sys`, the DOM, or Leptos rendering can be tested with standard `#[test]` and `cargo test`. This includes:

- Audio math helpers (`pitch_to_rate`, `extract_peaks`, EQ gain calculations)
- Route parsing (`Route::from_hash`)
- State machine logic
- BPM tap averaging
- Loop point clamping/validation

```rust
// src/audio/mod.rs (or a #[cfg(test)] block at the bottom of the file)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pitch_to_rate_at_zero() {
        assert!((pitch_to_rate(0.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn pitch_to_rate_plus_half() {
        assert!((pitch_to_rate(0.5) - 1.5).abs() < 0.001);
    }

    #[test]
    fn route_from_hash_settings() {
        assert_eq!(Route::from_hash("#/settings"), Route::Settings);
    }

    #[test]
    fn route_from_hash_unknown_defaults_to_main() {
        assert_eq!(Route::from_hash("#/anything"), Route::Main);
    }
}
```

Run with:
```bash
cargo test
```

---

### Layer 2: WASM / Headless Browser Tests (`#[wasm_bindgen_test]`)

For tests that need the DOM, the Leptos reactive runtime, or web APIs, use `wasm-bindgen-test` running in a real headless browser via `wasm-pack`.

#### Setup

**`Cargo.toml`:**
```toml
[dev-dependencies]
wasm-bindgen-test = "0.3"
```

**`.cargo/config.toml`** (at crate root — tells `cargo test` how to run WASM tests):
```toml
[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"
```

**Install `wasm-pack`** (one-time, manages chromedriver/geckodriver automatically):
```bash
cargo install wasm-pack
```

#### Writing WASM Tests

Tests live in `tests/wasm.rs` (or any `pub mod` — they must be public):

```rust
// tests/wasm.rs
use wasm_bindgen_test::*;

// Configure all tests in this file to run in a real browser (not Node.js)
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn signal_updates_correctly() {
    use leptos::prelude::*;

    let count = RwSignal::new(0i32);
    count.set(42);
    assert_eq!(count.get_untracked(), 42);
}
```

#### Testing That a Leptos Component Renders to the DOM

Mount the component into the document body and query the resulting DOM:

```rust
#[wasm_bindgen_test]
fn deck_label_shows_track_name() {
    use leptos::prelude::*;
    use leptos::mount::mount_to_body;
    use wasm_bindgen::JsCast;

    let track_name = RwSignal::new(Some("my_song.mp3".to_string()));

    // Mount the component into the real browser DOM
    let _handle = mount_to_body(move || {
        view! {
            <div id="test-label">
                {move || track_name.get().unwrap_or_default()}
            </div>
        }
    });

    // Query the DOM directly
    let doc = web_sys::window().unwrap().document().unwrap();
    let el = doc.get_element_by_id("test-label").unwrap();
    assert_eq!(el.text_content().unwrap(), "my_song.mp3");

    // Update signal and check reactivity
    track_name.set(Some("other_track.wav".to_string()));
    // Effects run async — use a microtask flush if needed (see async tests below)
}
```

#### Async WASM Tests

Tests that involve Promises, timers, or async signal flushing use `async fn`:

```rust
#[wasm_bindgen_test]
async fn reactive_update_is_reflected_in_dom() {
    use leptos::prelude::*;
    use leptos::mount::mount_to_body;
    use wasm_bindgen_futures::JsFuture;

    let value = RwSignal::new("before".to_string());

    let _handle = mount_to_body(move || {
        view! { <span id="async-test">{move || value.get()}</span> }
    });

    value.set("after".to_string());

    // Yield to the microtask queue so Leptos can flush reactive updates
    JsFuture::from(js_sys::Promise::resolve(&wasm_bindgen::JsValue::UNDEFINED))
        .await
        .unwrap();

    let doc = web_sys::window().unwrap().document().unwrap();
    let el = doc.get_element_by_id("async-test").unwrap();
    assert_eq!(el.text_content().unwrap(), "after");
}
```

#### Running the Tests

```bash
# Run all native tests (fast, no browser needed)
cargo test

# Run WASM tests in a headless Chrome browser
wasm-pack test --headless --chrome

# Run WASM tests in headless Firefox
wasm-pack test --headless --firefox

# Run both browsers
wasm-pack test --headless --chrome --firefox

# Run without --headless to open the browser and use DevTools to debug failures
wasm-pack test --chrome
```

#### What to Test at Each Layer

| Scenario | Layer |
|---|---|
| Route enum parsing | Native |
| Audio math (`pitch_to_rate`, peak extraction) | Native |
| Signal read/write behaviour | WASM (or Native for pure signal logic) |
| Leptos component renders correct initial HTML | WASM browser |
| Signal change updates DOM text/class | WASM browser (async) |
| `NodeRef` resolves after mount | WASM browser |
| Hash-change updates route signal | WASM browser |
| Web Audio `AudioContext` creation | ⚠️ Headless browsers may block audio — skip or mock |
| Canvas pixel output | ⚠️ Fragile — prefer visual regression tools instead |

#### Notes and Caveats

- **`AudioContext` in headless Chrome** requires the `--autoplay-policy=no-user-gesture-required` flag or a `webdriver.json` capabilities file. In practice, skip Web Audio integration tests in CI and test audio math with native unit tests instead.
- **Test isolation**: `mount_to_body` appends to the real `<body>`. If tests share a page, use unique IDs and clean up after each test with `on_cleanup` or by storing the mount handle (dropping it removes the component).
- **`wasm-pack test` vs `cargo test --target wasm32-unknown-unknown`**: `wasm-pack test` is the recommended approach because it automatically installs and manages the correct WebDriver binary version for your browser. The raw `cargo test --target` approach requires manually installing `wasm-bindgen-cli` at the exact matching version.
