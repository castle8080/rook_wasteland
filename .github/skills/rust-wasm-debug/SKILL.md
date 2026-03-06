---
name: rust-wasm-debug
description: Debug common Rust/WASM/Leptos issues. Use this when hitting compiler errors, runtime panics, silent failures, or unexpected behaviour in a Leptos CSR app.
---

## Panic Hook — Install First

Without this, WASM panics are silent or produce a useless "unreachable executed" message:

```rust
// In main() / #[wasm_bindgen(start)]
console_error_panic_hook::set_once();
```

Add `console_error_panic_hook = "0.1"` to `[dependencies]`.

---

## Common Issues and Fixes

### App starts silently / nothing renders

- Missing `#[wasm_bindgen(start)]` on `fn main()` in `lib.rs` — without it the WASM module loads but never calls `mount_to_body`.
- `trunk build` succeeded but `public_url` in `Trunk.toml` doesn't match the path you're serving from — assets 404.

### `RwSignal::new()` compile error about `Send + Sync`

Web-sys types (`AudioBuffer`, `AudioContext`, `HtmlCanvasElement`, etc.) are `!Send`. Use:
```rust
RwSignal::new_local(value)   // or signal_local()
```

### Event listener removes itself immediately

`gloo_events::EventListener` calls `removeEventListener` on drop. For app-lifetime listeners:
```rust
std::mem::forget(listener);  // intentional — keeps listener alive forever
```

### Canvas NodeRef is None on first render

The rAF loop setup must run *after* the first render so that `NodeRef` values are populated. Wrap the setup in `spawn_local`:
```rust
spawn_local(async move {
    // NodeRefs are now populated; safe to .get().expect(...)
});
```

### `on:input` handler: wrong event type

`on:input` gives you `web_sys::Event`, not `InputEvent`. Read the value with:
```rust
ev.target()
    .expect("input event has target")
    .unchecked_into::<HtmlInputElement>()
    .value()
```

### Effect doesn't fire after lazy node creation

Leptos `Effect`s fire when a signal *changes*. If a signal already had its current value when an audio/DOM node was created, the Effect won't re-fire. Fix: read current values with `.get_untracked()` and apply them at construction time.

### Two `on:` handlers can't share a closure

```rust
// Error: can't move closure into both handlers
// Fix:
let handler = Rc::new(move || { ... });
let h1 = handler.clone();
let h2 = handler.clone();
on:mouseup=move |_| h1()
on:mouseleave=move |_| h2()
```

### Clippy `new_without_default`

Any struct with `fn new()` must also impl `Default`:
```rust
impl Default for MyState {
    fn default() -> Self { Self::new() }
}
```

### `AudioBufferSourceNode` after stop

`AudioBufferSourceNode` is one-shot — it cannot be restarted. After `.stop()` it is inert. Store in `Option<AudioBufferSourceNode>` and create a fresh node on every `play()`.

### Web Audio API: user gesture required

`AudioContext::new()` will fail or be suspended without a prior user gesture (click, keydown, file input). Create and store the context lazily inside the first user-gesture handler, not at component initialization time.

### Spurious reactivity in rAF loop

Using `.get()` inside the rAF closure creates reactive subscriptions that fire on every signal change, potentially causing double-renders. Use `.get_untracked()` for reads inside the rAF loop.

### Two compilation targets diverge

`cargo clippy` targets the native rlib; `cargo clippy --target wasm32-unknown-unknown` targets the WASM cdylib. A warning can exist on one target but not the other. Always run the wasm32 target for CI:
```bash
cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

### `set_fill_style` deprecated

Use `set_fill_style_str("colour")` and `set_stroke_style_str("colour")` — not `set_fill_style(&JsValue)`.

### `save()` / `restore()` don't return Result

`CanvasRenderingContext2d::save()` and `restore()` return `()`. Don't `.expect()` them. `translate()`, `rotate()`, and `arc()` do return `Result<(), JsValue>`.
