---
name: leptos-component
description: Create a new Leptos 0.8 component following project conventions. Use this when asked to add a UI component, view, or reactive element.
---

## Component Checklist

### File placement
- UI components → `src/components/<name>.rs`
- Register in `src/components/mod.rs` with `pub mod <name>;`
- Full-page views → named `<Route>View` and shown/hidden via the `RwSignal<Route>` context in `app.rs` using `<Show when=...>`

### Routing
Do NOT use `leptos_router`. Navigation uses hand-coded hash-based routing. Routes are a plain enum in `src/routing.rs`; the current route is an `RwSignal<Route>` provided via context from the root `App`.

To navigate, use plain `<a>` tags:
```rust
<a href="#/my-route">"Go somewhere"</a>
```

Or imperatively:
```rust
web_sys::window()
    .expect("window exists")
    .location()
    .set_hash("#/my-route")
    .expect("set_hash is infallible");
```

To show a view only on a specific route, read the context signal:
```rust
let route = expect_context::<RwSignal<Route>>();
// In view!:
<Show when=move || route.get() == Route::MyRoute>
    <MyView/>
</Show>
```

### Component template

```rust
use leptos::prelude::*;

/// Brief description of what this component renders.
///
/// # Props
/// - `prop_name`: what it does
#[component]
pub fn MyComponent(
    /// Description of this prop
    value: i32,
    /// Optional label; defaults to None
    #[prop(optional)] label: Option<String>,
) -> impl IntoView {
    view! {
        <div class="my-component">
            // content
        </div>
    }
}
```

### Signal rules
- Use `RwSignal::new()` for `Send + Sync` types (primitives, enums, owned Strings)
- Use `RwSignal::new_local()` or `signal_local()` for `web-sys` types (`!Send`)
- Pass signals as props when a child needs to read OR write a parent's signal
- Use `provide_context` / `expect_context` for deeply nested access — avoid prop-drilling more than 2 levels

### Reactive values in view!
```rust
// Reactive (re-renders when signal changes)
{move || value.get()}

// One-time (renders once, not tracked)
{value.get_untracked()}

// Class toggle
class:active=move || is_active.get()

// Dynamic style
style:color=move || if error.get() { "red" } else { "inherit" }
```

### Event handlers
```rust
// Click
on:click=move |_| set_state.update(|s| s.toggle())

// Input — use web_sys::Event, not InputEvent
on:input=move |ev| {
    use web_sys::HtmlInputElement;
    use wasm_bindgen::JsCast;
    let val = ev.target()
        .expect("input event has target")
        .unchecked_into::<HtmlInputElement>()
        .value();
    set_text.set(val);
}
```

### When to use Show vs if-in-closure
- `<Show when=...>` — for toggling a single block lazily (preferred for large subtrees)
- `{move || if ... { view!{...}.into_any() } else { view!{...}.into_any() }}` — when you need two different subtrees with different types

### Doc comments
Every `pub` component, prop, and public function needs a `///` doc comment. Private helpers need one if the name alone isn't self-explanatory.

### Tests
If the component contains any pure logic (formatting, derived values, state transitions), extract it to a plain function and add a `#[cfg(test)]` unit test in the same file.
