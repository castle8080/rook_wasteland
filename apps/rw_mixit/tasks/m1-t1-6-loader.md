# Task T1.6: Create Async Audio File Loader

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Create `src/audio/loader.rs` with:
- `load_audio_file(file, deck, state, ctx)` — async function that reads a `File` to `ArrayBuffer`, decodes it via `AudioContext.decodeAudioData`, extracts waveform peaks, and updates reactive state signals.
- `read_file_as_array_buffer(file)` — private async helper wrapping `FileReader` in a `Promise`.
- `extract_peaks(buffer, num_columns)` — pure function that downsamples all channels to N peak values for waveform display.

---

## Design

### Data flow
```
web_sys::File
  → FileReader (via Promise) → ArrayBuffer
  → ctx.decode_audio_data(array_buffer) → AudioBuffer
  → extract_peaks → Vec<f32>
  → state.track_name.set(...)
  → state.duration_secs.set(...)
  → state.waveform_peaks.set(...)
  → deck.buffer = Some(audio_buffer)
```

### Function / type signatures
```rust
pub async fn load_audio_file(file: web_sys::File, deck: Rc<RefCell<AudioDeck>>, state: DeckState, ctx: AudioContext)
pub fn extract_peaks(buffer: &AudioBuffer, num_columns: usize) -> Vec<f32>
```

### Edge cases
- Empty buffer (`length == 0`) or `num_columns == 0`: returns `vec![0.0; num_columns]`.
- `samples_per_col` clamped to `.max(1)` to avoid division by zero.
- `get_channel_data(c)` returns `Result<Vec<f32>, JsValue>`; uses `unwrap_or_default()` which returns `Vec::default()` (empty) on error.

### Integration points
- Called via `spawn_local` in `on_file_change` handler in `deck.rs`.
- Writes to `DeckState` signals (requires `use leptos::prelude::Set` in scope).
- Stores decoded `AudioBuffer` in `AudioDeck.buffer` for playback in M2.

---

## Implementation Notes

- `Closure::<dyn FnMut(web_sys::ProgressEvent)>::new(...)` pattern used (not the shorthand) to satisfy the type checker.
- `leptos::prelude::Set` must be imported explicitly; `set()` on `RwSignal` is a trait method not in the default prelude export path in Leptos 0.8.
- `extract_peaks` is not tested on host (requires `AudioBuffer` which is a web-sys WASM type).

---

## Test Results

```
cargo test → 8 passed (deck.rs tests for format_duration/truncate_name)
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
trunk build → ✅ success
```
