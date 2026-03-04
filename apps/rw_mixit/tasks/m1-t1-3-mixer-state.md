# Task T1.3: Create MixerState

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Create `src/state/mixer.rs` with a `DeckId` enum (`A`/`B`) and a `MixerState` struct holding reactive signals for the crossfader, master volume, BPM values, and sync master selection.

---

## Design

### Data flow
`MixerState` will be created in `DeckView` (or a future App-level context) and passed to the `Mixer` component. Not yet wired up in M1 — defined here for forward compatibility.

### Function / type signatures
```rust
pub enum DeckId { A, B }
pub struct MixerState { ... }
impl MixerState { pub fn new() -> Self }
```

### Edge cases
- `DeckId` must be `Clone + PartialEq + Debug` for use as signal value.
- `Option<DeckId>` in `sync_master` is `Send + Sync` since `DeckId` has no non-Send fields.

### Integration points
- Re-exported from `src/state/mod.rs`.
- Will be used by `src/components/mixer.rs` in M5.

---

## Implementation Notes

Added `#[allow(dead_code)]` on enum, struct, and `impl` block because none are used yet in M1. Without this, clippy -D warnings reports errors.

---

## Test Results

```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
```
