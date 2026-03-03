# Task T1.2: Create DeckState

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Create `src/state/deck.rs` with a `DeckState` struct that holds all reactive signals for a single DJ deck: playback state, transport (position/duration), loop points, hot cues, EQ/filter parameters, FX toggles, VU level, and waveform peaks. All fields are `RwSignal<T>` from Leptos 0.8.

---

## Design

### Data flow
`DeckState` is created in `DeckView`, cloned, and passed to `Deck` as a prop. The `Deck` component passes it to `TrackLabel` and to the async file loader. Signals are read/written reactively.

### Function / type signatures
```rust
pub struct DeckState { ... }
impl DeckState { pub fn new() -> Self }
```

### Edge cases
- `[Option<f64>; 4]` for hot_cues: array of 4 elements, all `None` initially. This type is `Send + Sync + Copy + 'static` so works with `RwSignal`.
- `Option<Vec<f32>>` for waveform_peaks: `None` until a track is loaded.

### Integration points
- Used in `src/components/deck.rs` for the `Deck` and `TrackLabel` components.
- Written to in `src/audio/loader.rs` after decoding.

---

## Implementation Notes

Added `#[allow(dead_code)]` on the struct because most fields are not yet read in M1 (they'll be used in M2–M9). Without this, clippy -D warnings fails.

---

## Test Results

```
cargo test → 8 passed
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
```
