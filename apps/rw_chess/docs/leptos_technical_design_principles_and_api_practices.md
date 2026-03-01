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
- Use `leptos_meta` for `<head>` management, `leptos_router` for client-side routing

### Cargo.toml Feature Flags
```toml
[dependencies]
leptos = { version = "0.8", features = ["csr"] }
leptos_meta = { version = "0.8" }
leptos_router = { version = "0.8" }
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

## 8. Routing (leptos_router)

```rust
use leptos_router::{components::*, path};

view! {
    <Router>
        <Routes fallback=|| view! { <p>"Not Found"</p> }>
            <Route path=path!("/") view=Home />
            <Route path=path!("/game") view=GameView />
        </Routes>
    </Router>
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
