# Task T1.4: Create AudioContext Helper

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Create `src/audio/context.rs` with an `ensure_audio_context` function that lazily creates a shared `web_sys::AudioContext` on first call. The context is stored in `Rc<RefCell<Option<AudioContext>>>` so it can be shared between both decks without `Send + Sync` requirements.

---

## Design

### Data flow
`DeckView` creates the `Rc<RefCell<Option<AudioContext>>>` holder and passes a clone to each `Deck`. On first user gesture (Load Track click), `ensure_audio_context` is called, which creates the context and stores it. Subsequent calls return the existing context.

### Function / type signatures
```rust
pub fn ensure_audio_context(ctx: &Rc<RefCell<Option<AudioContext>>>) -> AudioContext
```
Returns a clone of the shared `AudioContext`.

### Edge cases
- `AudioContext::new()` must be called from a user gesture handler (browser security requirement). Calling it at page load would be silently suspended or blocked by the browser.
- `AudioContext` is `Clone` in web-sys (it's a JS object reference), so cloning is cheap.

### Integration points
- Called from `on_file_change` handler in `src/components/deck.rs`.
- The returned `AudioContext` is passed to `AudioDeck::new()` and `load_audio_file()`.

---

## Implementation Notes

Used `Rc<RefCell<Option<...>>>` instead of Leptos context because `AudioContext` is `!Send + !Sync` (a JS object), which Leptos 0.8's `provide_context` does not accept in its default (non-local) form.

---

## Test Results

```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
trunk build → ✅ success
```
