# Task T1.1: Update Cargo.toml for M1 Dependencies

**Milestone:** M1 — Audio Foundation & File Loading
**Status:** ✅ Done

---

## Restatement

Add `wasm-bindgen-futures = "0.4"` to `[dependencies]` (needed for `JsFuture` in the async file loader). Add `"OscillatorType"` to the `web-sys` features list (needed for flanger LFO oscillator type in `deck_audio.rs`).

---

## Design

### Data flow
Static configuration only. No runtime data flow.

### Function / type signatures
None — `Cargo.toml` change only.

### Edge cases
- `wasm-bindgen-futures` is a separate crate from `wasm-bindgen`; it must be listed explicitly.
- `OscillatorType` (the enum) is distinct from `OscillatorNode` (the interface); both must be listed as separate web-sys features.

### Integration points
- `src/audio/loader.rs` uses `JsFuture` from `wasm_bindgen_futures`.
- `src/audio/deck_audio.rs` uses `OscillatorType` (via `BiquadFilterType` pattern).

---

## Implementation Notes

Added `wasm-bindgen-futures = "0.4"` after `js-sys` in `[dependencies]`. Added `"OscillatorType"` on the same line as `"OscillatorNode"` in the web-sys features list.

---

## Test Results

**Automated:**
```
cargo clippy --target wasm32-unknown-unknown -- -D warnings → 0 errors
trunk build → ✅ success
```

---

## Callouts / Gotchas

- `OscillatorType` is needed even if you only call `flanger_lfo.start()` without setting the type explicitly, because `deck_audio.rs` imports it from web-sys.
