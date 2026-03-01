# Principles and Lessons - RW Defender

Accumulated knowledge about building this Rust/WASM game correctly.

## Rust + WASM Best Practices

### Setup
- Use `crate-type = ["cdylib"]` in `Cargo.toml` for WASM libraries (NOT `rlib` or binary).
- Target `wasm32-unknown-unknown` via `trunk build` or `wasm-pack build --target web`.
- `trunk serve` handles the full dev workflow (build, serve, hot reload).
- Add `console_error_panic_hook::set_once()` in `#[wasm_bindgen(start)]` for readable panic messages.

### API Lessons

#### web-sys Canvas API
- Use `set_fill_style_str(&str)` NOT `set_fill_style(&JsValue)` — the `_str` variant was added in newer versions and avoids allocation.
- Use `set_stroke_style_str(&str)` similarly.
- `ctx.fill_rect(x, y, w, h)` — all f64.
- `ctx.set_image_smoothing_enabled(false)` — essential for crisp pixel art.
- `ctx.set_font("12px monospace")` before `fill_text`.

#### wasm-bindgen
- Event listener closures must be `Box<dyn FnMut(EventType)>` wrapped in `Closure::wrap`.
- Call `.forget()` on closures passed to `add_event_listener_with_callback` to prevent them being dropped.
- Use `thread_local! { static X: RefCell<T> }` for game state accessible from multiple closures.
- `Rc<RefCell<Option<Closure<...>>>>` pattern for recursive RAF callbacks.

#### Random Numbers
- Use `rand = "0.8"` with `getrandom = { version = "0.2", features = ["js"] }` for WASM entropy.
- `SmallRng::seed_from_u64(42)` for a fast, seedable RNG inside the game struct.

### Performance
- Pre-generate all sprites at startup (`SpriteAtlas::new()`) — never allocate sprites in the hot path.
- Skip transparent pixels (alpha < 10) in `Sprite::draw` to reduce fill_rect calls.
- Batch same-color pixels when possible.
- Cap delta time at 100ms to prevent "spiral of death" after tab focus loss.
- Use `Vec::with_capacity` for entity lists to avoid frequent reallocations.
- Mark entities `active = false` instead of removing from vec mid-frame to avoid borrow issues.
- Purge inactive entities between frames (retain) but NOT mid-collision detection.

### Architecture
- Use composition over inheritance for entities (single `Entity` struct with `EntityType` enum).
- Keep rendering and game logic strictly separated (pass `&Renderer` into `render`).
- `InputState::consume_*` pattern for one-shot keys (pause, start) prevents double-firing.
- Delta time in SECONDS (divide JS timestamp ms by 1000.0).

### Code Quality
- Run `cargo clippy -- -D warnings` before commits.
- Use `#[rustfmt::skip]` for sprite pattern arrays to keep them readable.
- Keep all sprite pixel data as `u8` palette indices — convert to `u32` ARGB only in `new()`.
- Color format: `0xAARRGGBB` (alpha high byte, then R, G, B).

### Common Compile Errors
- `Closure::wrap` requires the inner type to be `FnMut`, not `Fn`.
- `as_ref().unchecked_ref()` is needed to convert `Closure` to `&Function`.
- WASM doesn't have threads — use `thread_local!` not `static mut` or `lazy_static`.
- `wasm_bindgen(start)` function must be `pub fn` with no arguments.
- `web_sys` features must be explicitly listed in `Cargo.toml` — missing features = compile error.

### Testing
- Pure logic (math, collision, entity) can be tested with native `cargo test` (no WASM needed).
- WASM-specific tests use `wasm-bindgen-test` and run with `wasm-pack test --chrome --headless`.
- Keep WASM-independent logic in `utils/`, `entities/`, `graphics/` for easy native testing.
