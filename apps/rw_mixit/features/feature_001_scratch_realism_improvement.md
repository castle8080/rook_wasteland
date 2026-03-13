# Feature 001 — Scratch Realism Improvement

## Status
Implemented

## Summary
The current scratch simulation maps angular velocity linearly to `playbackRate`, causing even moderate wrist-flick gestures to overshoot into unnaturally high pitch territory (3–4× speed). This feature recalibrates the sensitivity using a non-linear (square-root) curve, adds pre-computed reverse-buffer playback so backward drags sound like the record playing in reverse, and applies short per-update smoothing to suppress event-rate jitter — together producing a convincing baby-scratch feel.

## Problem Statement
DJs performing a baby scratch expect the forward push to raise pitch modestly (1.0–2.0× range) and the backward drag to sound like reverse playback of the music. Currently, a 30° drag in 50 ms produces a rate of ~3×, which sounds more like a high-pitched chipmunk than a vinyl scratch. Additionally, backward drags produce silence (rate clamped to 0.0) rather than reverse audio, which completely breaks the illusion of holding and moving a record. The combination makes the feature feel like a pitch-bend slider rather than a platter.

## Goals
- A typical wrist-flick baby scratch (30°–90° arc, moderate speed) produces playback rates in the 0.5–2.0× range on the forward stroke.
- Backward drags produce audible reverse playback proportional to drag speed.
- Rate transitions between forward and reverse feel seamless with no audible click or pop.
- Release of the platter snaps back to normal speed within ~100 ms (current behavior preserved).
- All existing unit tests continue to pass; new unit tests cover the recalibrated formula and buffer-swap logic.

## Non-Goals
- This feature will NOT add a crossfade/blend buffer to smooth the forward↔reverse buffer swap at the audio sample level (a phase-coherent morph); a clean swap is sufficient.
- This feature will NOT change the stutter, echo, reverb, or flanger effects.
- This feature will NOT change the platter canvas visual (rotation animation, label rendering, etc.).
- This feature will NOT persist any per-user sensitivity preference or expose a sensitivity knob in the UI (v1 uses a fixed calibrated constant).
- This feature will NOT support simultaneous scratch on both decks at the same time (each deck is independent, which is already the case).

## User Stories
- As a user performing a baby scratch, I want a forward push on the platter to raise the pitch slightly (not dramatically) so that it sounds like I'm speeding up the record with my hand.
- As a user performing a baby scratch, I want a backward drag on the platter to play the music in reverse so that the full "wicky-wicky" sound is audible.
- As a user releasing the platter after a scratch, I want the record to quickly spin back up to normal speed so that the music resumes naturally.
- As a user, I want the pitch change to feel proportional to how fast I drag — a slow drag should produce a subtle pitch shift, a fast flick should produce a more dramatic one, but never spike into chipmunk territory.

## Functional Requirements
1. The sensitivity curve for `scratch_rate_from_angular_velocity()` shall use a non-linear mapping (square-root of the normalized angular velocity) so that the rate at a "moderate" drag (≈ TAU rad/s, ~1 rotation/second) is approximately 1.0–1.3×, and the maximum forward rate at 4× clamping requires at least 4× the angular velocity needed for 1.0× in the current implementation.
2. When a scratch gesture begins (`scratch_start`), the system shall pre-check whether a reversed `AudioBuffer` exists for the current track; if not, it shall defer to forward-only behavior (existing v1 behavior) without error.
3. During `scratch_move`, when the unwrapped `d_angle` is negative (backward drag), the system shall stop the current forward `AudioBufferSourceNode` (without scheduling a ramp) and start a new `AudioBufferSourceNode` using the reversed buffer at the reverse-equivalent seek position, with a positive `playbackRate` proportional to the magnitude of the negative angular velocity.
4. During `scratch_move`, when the unwrapped `d_angle` transitions back to positive (forward drag) after a reverse phase, the system shall stop the reversed source and resume the forward source at the correct seek position.
5. Each `pointermove` rate update shall use `linearRampToValueAtTime` over a 10–15 ms window (rather than `set_value` directly) to prevent pops from high-frequency event delivery; this smoothing shall only apply during forward-buffer mode.
6. `scratch_end` shall behave identically to the current implementation: anchor current rate + linear ramp to `pre_scratch_rate` over ~100 ms.
7. The reversed `AudioBuffer` shall be computed once per file load, immediately after `decode_audio_data` returns, by iterating channel data in reverse order. It shall be stored in `AudioDeck` alongside the forward buffer.
8. If the reversed buffer computation fails (e.g., zero-length audio, OOM), the system shall log a warning and fall back to forward-only scratch without panicking.

## UI / UX Notes
- No changes to the FX panel layout, toggle buttons, or knobs. The scratch toggle behavior is identical.
- No new visual controls are needed. The improved behavior is purely in the audio response.
- The platter canvas animation (visual rotation) is driven by the platter animation loop independently of playbackRate and requires no changes.

## Architecture Fit

### Existing modules / components touched
- **`src/audio/deck_audio.rs`** — primary change site:
  - `AudioDeck` struct: add `reversed_buffer: Option<AudioBuffer>` field alongside existing `buffer: Option<AudioBuffer>`.
  - `load_audio()` (or wherever `decode_audio_data` completes): add a call to `compute_reversed_buffer()` to populate `reversed_buffer`.
  - `scratch_rate_from_angular_velocity()`: replace linear formula with square-root non-linear curve; update unit tests.
  - `scratch_start()`: snapshot current forward playback position for use in buffer swap logic.
  - `scratch_move()`: add direction-detection logic; implement forward↔reverse source swap using new helper methods.
  - `scratch_end()`: no change expected.
- **`src/state/deck.rs`** — no changes anticipated (scratch state signals already exist).
- **`src/components/deck.rs`** — no changes anticipated (pointer event handlers are correct).

### New modules / helpers introduced
- `compute_reversed_buffer(ctx: &AudioContext, forward: &AudioBuffer) -> Result<AudioBuffer, JsValue>` — pure function, added to `deck_audio.rs`; can be tested natively with mock data.
- `scratch_rate_nonlinear(normalized_velocity: f64) -> f32` — extracted pure function replacing the current linear formula; unit-testable natively.

### State shape changes
- `AudioDeck` gains one new field: `pub reversed_buffer: Option<AudioBuffer>`.
- `AudioDeck` scratch state gains: `pub scratch_in_reverse: bool` to track whether the current scratch phase is using the reverse buffer (used in `scratch_move` direction-change detection).

### Persistence
None. `reversed_buffer` is computed in-memory at file load time and discarded when a new file is loaded (same lifecycle as `forward buffer`).

## Open Questions

All open questions resolved before implementation:

1. **Seek position during buffer swap**: Add `scratch_position_secs: f64` to `AudioDeck`. Set to `current_position()` at `scratch_start()`. Integrate `rate * dt` each `scratch_move()` call (positive when forward, negative when reverse). Use this field for all offset calculations on buffer swap. When swapping to reversed buffer: `rev_offset = (duration - scratch_position_secs).clamp(0, duration)`. When swapping back to forward: `fwd_offset = scratch_position_secs.clamp(0, duration)`.
2. **Rate smoothing and reverse**: Apply the 12 ms `linearRampToValueAtTime` smoothing during forward phases only. During reverse phases, use `set_value()` directly (immediate) to prevent smoothing from blurring the sharp transient edges that define the scratch sound.
3. **Non-linear curve constant**: `SCRATCH_SENSITIVITY = 0.9`. For TAU rad/s (1 rotation/second): `normalized_vel = 1/0.55 ≈ 1.818`, rate = `sqrt(1.818) * 0.9 ≈ 1.21`. This lands in the 1.0–1.3 target range. `SCRATCH_RATE_MAX = 3.5` (slightly below the old 4.0 clamp to reinforce the less-aggressive feel).
4. **Memory overhead**: A 5-minute stereo track at 44.1 kHz (float32) costs ~53 MB per buffer. The reversed buffer adds another ~53 MB per deck (106 MB total per deck). Acceptable — rw_mixit targets desktop browsers with ≥8 GB RAM and typical tracks are 3–7 minutes.

## Out of Scope / Future Work
- **Tunable sensitivity knob in the FX panel**: users could dial in their preferred scratch sensitivity. Deferred to a future feature.
- **Phase-coherent forward↔reverse morph**: crossfade between forward and reversed buffer at the transition point to avoid any audible click on direction change. Potentially needed if the clean swap is noisy.
- **Torque simulation / inertia**: a realistic model where the virtual record has angular momentum and "resists" sudden direction changes. High complexity, deferred.
- **Needle drop sound**: a brief vinyl crackle/thump when scratch mode is first activated. A nice polish touch, deferred.

---
<!-- The sections below are filled in during the implementation phase -->

## Implementation Plan

### Files Modified

**`src/audio/deck_audio.rs`** — Primary change site:
- Replaced `scratch_rate_from_angular_velocity()` (linear) with `scratch_rate_nonlinear(normalized_vel: f64) -> f32` (square-root curve)
- Added 3 named constants: `SCRATCH_SENSITIVITY = 0.9`, `SCRATCH_RATE_MAX = 3.5`, `SCRATCH_SMOOTH_SECS = 0.012`
- Added `compute_reversed_buffer(ctx, forward) -> Result<AudioBuffer, JsValue>` free function
- Extended `AudioDeck` struct with 4 new fields: `reversed_buffer`, `scratch_in_reverse`, `scratch_position_secs`, `scratch_was_playing`
- Rewrote `scratch_start()`: saves `scratch_was_playing`, initializes `scratch_position_secs` from `current_position()`
- Rewrote `scratch_move()`: direction detection, buffer swap (fwd↔rev), position integration, 12 ms forward-only smoothing
- Rewrote `scratch_end()`: always stops source, resumes via `play()` if was_playing, anchors offset otherwise
- Replaced 5 native scratch tests with 5 new `scratch_nonlinear_*` tests; added 6 new WASM tests

**`src/audio/loader.rs`** — File load pipeline:
- After `deck.buffer = Some(audio_buffer)`, calls `compute_reversed_buffer` and stores result or logs warning (non-fatal)

### Key Architectural Decisions

- Position is integrated in `scratch_move` using `scratch_in_reverse` (actual buffer state) not `going_reverse` (user intent), so the fallback path (no reversed buffer, backward drag) correctly increments position instead of decrementing it
- `scratch_end` uses `play()` to resume (rather than re-anchoring `started_at` directly), which keeps `current_position()` accurate for the waveform rAF loop

## Spec Changes

- **`doc/rw_mixit_tech_spec.md` §8.12**: Completely rewritten to describe the non-linear curve, reversed buffer swap, position integration, and loader trigger. Updated "Dropped" table to mark true-reverse scratch as Implemented.
- **`doc/implementation_plan.md`**: M9 all tasks marked ✅ Done (done in a prior session).
- **`doc/rw_mixit_spec.md`**: No changes needed — feature aligns with existing spec goals.
- **`doc/rw_mixit_tech_spec.md` §11 "Dropped"**: True reverse scratch row updated from "deferred" to "Implemented (Feature 001)".

## Test Strategy

### Tier 1 (native, `cargo test`)
- `scratch_nonlinear_stationary_gives_zero` — rate = 0 at zero velocity
- `scratch_nonlinear_one_rotation_per_second_in_target_range` — 1 rot/s → rate ∈ [1.0, 1.3]
- `scratch_nonlinear_clamped_at_upper_bound` — massive velocity → SCRATCH_RATE_MAX
- `scratch_nonlinear_sqrt_is_sublinear` — doubling velocity < doubles rate
- `scratch_nonlinear_half_speed` — 0.5 rot/s produces expected half-speed rate

### Tier 2 (WASM, wasm-pack)
- `reversed_buffer_is_none_before_load` — initial state
- `scratch_in_reverse_starts_false` — initial state
- `reversed_buffer_has_same_dims` — channel/length/rate preserved
- `reversed_buffer_reverses_content` — sample order is reversed
- `reversed_buffer_both_channels_reversed` — stereo both channels
- `scratch_on_paused_deck_does_not_panic` — safe on paused deck

### Coverage Gaps
- Backward drag producing actual reversed sound — requires full decode pipeline; manual smoke test only
- Rate subjective feel — human evaluation only

## Decisions Made

### Decision: Square-root curve with sensitivity constant 0.9
**Chosen:** `rate = sqrt(normalized_vel) * 0.9`, where `normalized_vel = |d_angle|/dt / (TAU * 0.55)`
**Alternatives considered:** Linear (existing), cubic root, logarithmic
**Rationale:** Square-root is the natural "compressive" choice — it strongly reduces very-high-velocity rates while preserving moderate ones. The `0.9` constant is calibrated so 1 rotation/sec maps to ≈1.21×, firmly in the target 1.0–1.3 range.

### Decision: Pre-compute reversed buffer on file load (not on first scratch start)
**Chosen:** Compute reversed buffer in `loader.rs` immediately after decode; store in `AudioDeck.reversed_buffer`
**Alternatives considered:** Lazy-compute on first backward drag; compute in a Web Worker
**Rationale:** File load is already async and the computation is fast (linear copy + in-place reverse). Doing it at load time keeps `scratch_move` synchronous and branch-free.

### Decision: Use `scratch_in_reverse` (not `going_reverse`) for position integration
**Chosen:** Integrate position using the actual buffer state (`scratch_in_reverse`)
**Alternatives considered:** Integrate using user intent (`going_reverse`, i.e. `d_angle < 0`)
**Rationale:** When no reversed buffer is available, backward drag (`going_reverse=true`) leaves the forward buffer playing forward. Using `going_reverse` for integration would subtract position while audio moves forward, diverging the estimate from reality. `scratch_in_reverse` correctly tracks what is actually playing. Surfaced by code review.

### Decision: 12 ms smoothing only during forward phase
**Chosen:** Forward: `linearRampToValueAtTime(rate, now + 0.012)`; Reverse: `set_value(rate)` immediately
**Alternatives considered:** Smooth both directions; no smoothing at all
**Rationale:** High-frequency pointermove events cause audible pops during forward-buffer playback due to step discontinuities. For reverse scratch, sharp transient edges define the sound — smoothing would blur the "wicky" character.

## Lessons / Highlights

### Reversed AudioBuffer position mapping requires care
When swapping from forward→reversed buffer at position T, the correct reversed-buffer offset is `duration - T` (distance from the *end* of the track, not the beginning). This is because the reversed buffer plays from index 0 (which corresponds to the last frame of the original) forward. Getting this backwards produces a jump to the wrong position in the track.

### Position integration should track actual playback state, not gesture direction
During a scratch session, if `reversed_buffer` is unavailable, the forward buffer continues playing forward even on a backward drag gesture. The position integrator must branch on `scratch_in_reverse` (what is actually playing) rather than `going_reverse` (what the user's hand is doing) to stay accurate.

### AudioBuffer.clone() is a cheap JS reference copy
In web-sys, `AudioBuffer` is a JS reference type. `.clone()` copies the JS object handle, not the PCM data (~53 MB). Use freely to satisfy the borrow checker when passing `self.buffer` or `self.reversed_buffer` into expressions that also borrow `self` mutably.

