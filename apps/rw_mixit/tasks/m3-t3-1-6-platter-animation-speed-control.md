# Task T3.1–T3.6: Platter Animation & Speed Control

**Milestone:** M3 — Platter Animation & Speed Control
**Status:** 🔄 In Progress

---

## Restatement

Implement the animated vinyl platter canvas and the pitch/tempo fader.
`draw_platter` renders a cartoon vinyl disc (dark background, concentric groove
rings, center label with track name, spindle dot) onto a per-deck `<canvas>`
every rAF frame. The groove rings rotate based on accumulated playback time and
rate, making the record look like it is spinning. A tonearm line sweeps inward
from the outer groove to near the label over the full track duration. The pitch
fader is an `<input type="range">` that writes to `DeckState.playback_rate` via
`pitch_to_rate()`. A Leptos `Effect` propagates `playback_rate` changes to the
live `AudioBufferSourceNode.playbackRate` AudioParam while a source is active.

**Out of scope:** scratch simulation (M9), VU meter animation (future), BPM
display (M4), mixer crossfader (M5).

---

## Design

### Data flow

1. `DeckState.current_secs` (updated by rAF loop) + `DeckState.playback_rate` →
   `draw_platter` reads both via `.get_untracked()` each frame to compute groove
   rotation angle and tonearm progress.
2. User drags pitch fader → `on:input` handler → `state.playback_rate.set(pitch_to_rate(raw))`
   → reactive Effect fires → `source.playback_rate().set_value(rate)`.

### Function / type signatures

```rust
/// Draw the platter for one rAF frame.
pub fn draw_platter(canvas_ref: &NodeRef<html::Canvas>, state: &DeckState, deck_side: &str)

/// Convert pitch fader (−1.0..+1.0) → playback rate multiplier.
pub fn pitch_to_rate(fader: f64) -> f64

/// Convert playback rate → pitch fader (inverse; for slider initialization).
pub fn rate_to_pitch(rate: f64) -> f64

/// Pitch / tempo fader component.
#[component] pub fn PitchFader(state: DeckState) -> impl IntoView
```

### Edge cases

- No track loaded: `draw_platter` shows empty spinning disc (no label text).
- `duration_secs == 0`: tonearm stays at start angle (progress clamps to 0).
- Pitch fader changes while not playing: `Effect` runs but `source` is `None`; no-op.
- Rapid fader movement: `Effect` fires on each signal write; AudioParam `set_value` is idempotent.

### Integration points

- `src/canvas/raf_loop.rs` — `start_raf_loop` gains two new `NodeRef<html::Canvas>` params.
- `src/components/deck.rs` — `DeckView` creates platter refs; `Deck` renders platter canvas,
  `PitchFader`, and hosts the playback_rate Effect.
- `src/canvas/mod.rs` — expose new `platter_draw` module.
- `src/components/mod.rs` — expose new `pitch_fader` module.

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Groove rotation angle grows unbounded (float precision) | For typical tracks (< 2 hr), `current_secs * 0.55 * TAU` stays well within f64 precision. No issue in practice. |
| Simplicity | Could cache the static disc shape offscreen like waveform | Not worth it — platter is redrawn every frame anyway due to rotation. Profiling first. |
| Coupling | rAF loop signature grows with every new canvas | Acceptable for this project scope; six parameters total. |
| Performance | `arc()` called 18 times per frame per deck = 36 calls | At 60 fps, 36 simple arc calls is negligible. Canvas API overhead not a bottleneck here. |
| Testability | `draw_platter` requires DOM/canvas — not unit-testable | Pure functions (`pitch_to_rate`, `rate_to_pitch`, `truncate_label`, tonearm math) are fully unit-testable. |

---

## Implementation Notes

- Groove rotation angle: `angle = current_secs * (33.0/60.0) * playback_rate * TAU`
- Tonearm pivot: `(cx + r * 1.0, cy − r * 0.9)` — upper-right, at canvas edge
- Tonearm arm length: `r * 0.907`; start angle: `1.682 rad ≈ 96.3°`; max sweep: `π/4 (45°)`
- Geometry verified: start tip lands at outer groove (~90% of r), end tip at ~44% of r
  (just outside label at 35% of r).

---

## Test Results

**Automated:**
```
cargo test output — see build step
```

**Manual steps performed:**
- [ ] Load a track on Deck A — platter draws with label text visible
- [ ] Press Play — grooves spin at correct 33 RPM rate
- [ ] Drag pitch fader right — platter spins faster; audio pitch/speed rises
- [ ] Drag pitch fader left — platter spins slower; audio pitch/speed drops
- [ ] Tonearm sweeps from right side toward center as track plays

---

## Review Notes

---

## Callouts / Gotchas

- `ctx.save()`/`ctx.translate()`/`ctx.rotate()`/`ctx.restore()` MUST be balanced.
  An unbalanced save/restore leaks the transform matrix across frames.
- The `Effect` for `playback_rate → AudioParam` will fire once on mount with the
  default rate (1.0) and source = None. That is a safe no-op.
