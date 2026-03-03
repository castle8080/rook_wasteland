# Task T2.1–T2.3: AudioDeck Playback Methods

**Milestone:** M2 — Playback & Waveform
**Status:** 🔄 In Progress

---

## Restatement

Add `play`, `pause`, `stop`, `current_position`, `seek`, and `cue` methods to `AudioDeck`
in `src/audio/deck_audio.rs`. `AudioBufferSourceNode` is one-shot, so `play` creates a new
source node each call, connects it to `pre_gain`, and starts it at the given offset.
`pause` records the current position and stops the source. `seek` optionally restarts.
A `cue_point: Option<f64>` and `pre_nudge_rate: Option<f32>` field are added for later tasks.
Also connects `analyser → destination` so audio is actually audible in M2
(this direct connection will be replaced by the crossfader path in M5).

---

## Design

### Data flow

Play: button click → controls.rs → `audio_deck.borrow_mut().play(offset, rate)` →
new `AudioBufferSourceNode` → connect to `pre_gain` → audio chain → `analyser` → `destination`.

Pause: button click → `audio_deck.borrow_mut().pause()` → records position → source.stop() →
position returned and written to `state.current_secs`.

### Function / type signatures

```rust
pub fn play(&mut self, offset: f64, rate: f32)
pub fn pause(&mut self) -> f64           // returns position held
pub fn stop(&mut self)
pub fn current_position(&self) -> f64
pub fn seek(&mut self, position: f64, rate: f32)
pub fn cue(&mut self, rate: f32)         // set or jump to cue_point
pub fn nudge_start(&mut self, direction: f32)  // direction: +1.0 or -1.0
pub fn nudge_end(&mut self)
```

New fields on `AudioDeck`:
- `cue_point: Option<f64>` — set by cue()
- `pre_nudge_rate: Option<f32>` — saved before nudge

### Edge cases

- `play()` called when no buffer: early return.
- `pause()` called when not playing: returns `offset_at_play`.
- `current_position()` when `started_at` is None: returns `offset_at_play`.
- `seek()` past end of track: clamped to `[0, buffer.duration()]`.

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Must stop existing source before creating new one in play() | Done; stop() clears source |
| Simplicity | Could pass rate from DeckState directly vs. as parameter | Parameter is cleaner — caller reads signal, AudioDeck stays pure |
| Coupling | Direct analyser→destination bypasses future crossfader (M5) | Comment the connection clearly; M5 will disconnect and re-route |
| Performance | Creating new AudioBufferSourceNode on each play() is standard Web Audio practice | No issue — this is the correct API pattern |
| Testability | Web Audio node methods only work in browser | Unit tests cover the pure math (current_position); WASM tests cover node creation |

---

## Implementation Notes

- `start_with_when_and_grain_offset(0.0, offset)` is the web-sys name for `start(when, offset)`.
- `AudioContext::destination()` is available on the stored `ctx` field.
- `linear_ramp_to_value_at_time` signature in web-sys takes `(f64, f64)` — rate as f64.

---

## Test Results

**Automated:**
(filled after implementation)

**Manual steps performed:**
- [ ] Load a track, press Play — audio plays
- [ ] Press Pause — audio stops, position held
- [ ] Press Play — resumes from held position
- [ ] Press Stop — resets to beginning
- [ ] Cue button works

---

## Review Notes

---

## Callouts / Gotchas
