# Task T1.5: Create AudioDeck Node Graph

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Create `src/audio/deck_audio.rs` with an `AudioDeck` struct that owns the entire Web Audio node graph for one deck: pre-gain, 3-band EQ (high-shelf, peaking, low-shelf), sweep filter, reverb (convolver with dry/wet bypass), echo (delay + feedback loop), flanger (delay + LFO), channel gain, and analyser. The `AudioDeck::new()` constructor builds and wires all nodes, then wraps the result in `Rc<RefCell<AudioDeck>>`.

---

## Design

### Data flow
```
source → pre_gain → eq_high → eq_mid → eq_low → sweep_filter
       → reverb_dry ──────────────────────────────────────→ echo_dry → channel_gain → analyser
       → reverb ────→ reverb_wet ──────────────────────────→ echo_dry
                                         echo_dry → echo_delay → echo_wet → channel_gain
                                                  ↘ echo_feedback (loop)
       → flanger_delay → flanger_wet → channel_gain
         (lfo → flanger_depth → flanger_delay.delay_time)
```

### Function / type signatures
```rust
pub struct AudioDeck { ... }
impl AudioDeck { pub fn new(ctx: AudioContext) -> Rc<RefCell<AudioDeck>> }
```

### Edge cases
- `flanger_lfo.start()` is called in the constructor; it runs at 0.5 Hz but flanger_wet gain is 0.0, so there's no audible effect until M7 enables it.
- `analyser → destination` is NOT wired here — that connection is deferred to M5 (master gain / routing milestone).
- `reverb` (ConvolverNode) has no impulse response buffer set — it will pass audio through dry by default.

### Integration points
- `deck.buffer` is set by `load_audio_file` in `loader.rs`.
- `deck.source` will be created/replaced each time playback starts (M2).
- `deck.analyser` will be read by the waveform canvas component (M3).

---

## Implementation Notes

Added `#[allow(dead_code)]` on the struct because most fields are not yet used in M1. The LFO oscillator is started immediately (required before connecting) with wet gain at 0.

---

## Test Results

```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
trunk build → ✅ success
```
