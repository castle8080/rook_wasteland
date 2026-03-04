# Task T1.7: Replace Deck Component with Load Track UI

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Replace the placeholder `DeckPlaceholder`/`DeckView` in `src/components/deck.rs` with a real implementation:
- `DeckView` — creates the shared `AudioContext` holder and two `DeckState` instances; renders `[Deck A] [Mixer] [Deck B]`.
- `Deck` — receives `side`, `DeckState`, and `Rc<RefCell<Option<AudioContext>>>` as props; renders a hidden file input, a "Load Track" button, and a `TrackLabel`.
- `TrackLabel` — displays the truncated track name and formatted duration from reactive signals.
- `format_duration(secs: f64) -> String` — formats seconds as `"M:SS"` or `"--:--"` for zero.
- `truncate_name(name: &str, max_len: usize) -> String` — truncates with `…` if longer than max_len.

---

## Design

### Data flow
```
User clicks "Load Track"
  → on_load_click: hidden <input type="file">.click()
  → Browser file picker
  → on_file_change: ensure_audio_context, create AudioDeck if needed
  → spawn_local(load_audio_file(...))
  → state.track_name / duration_secs / waveform_peaks updated reactively
  → TrackLabel re-renders
```

### Function / type signatures
```rust
#[component] pub fn DeckView() -> impl IntoView
#[component] pub fn Deck(side: &'static str, state: DeckState, audio_ctx_holder: Rc<RefCell<Option<AudioContext>>>) -> impl IntoView
#[component] pub fn TrackLabel(state: DeckState) -> impl IntoView
fn truncate_name(name: &str, max_len: usize) -> String
fn format_duration(secs: f64) -> String
```

### Edge cases
- `NodeRef<leptos::html::Input>` is `Copy`, so no `.clone()` needed in the click handler.
- `AudioDeck` is created lazily on first file load (not at component mount) because `AudioContext::new()` requires a user gesture.
- `audio_ctx_holder` is passed as a prop (not via `provide_context`) because `Rc<RefCell<Option<AudioContext>>>` is `!Send + !Sync`.

### Integration points
- Calls `ensure_audio_context` from `src/audio/context.rs`.
- Calls `AudioDeck::new` from `src/audio/deck_audio.rs`.
- Calls `load_audio_file` from `src/audio/loader.rs`.
- Reads `DeckState` signals from `src/state/deck.rs`.

---

## Test Results

```
cargo test → 8 passed
  - test_format_duration_zero
  - test_format_duration_values (125.0→"2:05", 60.0→"1:00", 3661.0→"61:01")
  - test_truncate_name_short
  - test_truncate_name_long
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
trunk build → ✅ success
```

---

## Callouts / Gotchas

- The hidden file `<input>` approach avoids custom styling of the native file picker. The "Load Track" button triggers `.click()` on the hidden input via `NodeRef`.
- `leptos::task::spawn_local` (not `wasm_bindgen_futures::spawn_local`) is the correct Leptos 0.8 API for spawning async tasks from event handlers.
