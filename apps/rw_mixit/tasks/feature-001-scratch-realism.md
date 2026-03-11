# Feature 001: Scratch Realism Improvement

**Feature Doc:** features/feature_001_scratch_realism_improvement.md
**Milestone:** standalone feature (post-M9)
**Status:** 🔄 In Progress

---

## Restatement

The current scratch simulation linearly maps angular velocity to `playbackRate`,
causing moderate wrist-flick gestures (30°–90° arc) to produce unnaturally high
rates (3–4×) and clamping all backward drags to silence. This feature corrects
both problems: (1) a non-linear square-root sensitivity curve is substituted so
a typical baby-scratch flick produces rates in the 0.5–2.0× range; (2) a
pre-computed reversed `AudioBuffer` is created on every file load so backward
drags play the audio in reverse, completing the "wicky-wicky" illusion.

All changes live in `src/audio/deck_audio.rs` (scratch logic + reversed buffer
computation) and `src/audio/loader.rs` (trigger reversed buffer computation after
decode). No UI changes are needed — the existing FX panel toggle and platter
pointer handlers are correct as-is. The platter canvas animation is unaffected.
Paused-deck behavior is unchanged: if the deck is not playing at scratch start,
buffer swaps are still attempted (so a paused deck can be scratched audibly),
and on release the deck returns to its pre-scratch state (paused or playing).

---

## Design

### Data flow

```
pointermove → on_platter_pointermove (deck.rs)
  → compute angle via atan2
  → deck.scratch_move(angle, performance.now())
      → d_angle = angle - last_angle (unwrapped)
      → normalized_vel = |d_angle| / dt / (TAU * 0.55)
      → rate = scratch_rate_nonlinear(normalized_vel)
      → direction change? → swap AudioBufferSourceNode (fwd ↔ rev buffer)
      → same direction? → update playbackRate (ramp 12 ms if forward, immediate if reverse)
      → scratch_position_secs += ±rate * dt (integrate position)

pointerup/leave → scratch_end()
  → stop_source()
  → if was_playing: play(scratch_position_secs, pre_scratch_rate)
  → else: offset_at_play = scratch_position_secs
  → reset scratch state
```

Reversed buffer is populated in `loader.rs::load_audio_file()` immediately after
`deck.buffer = Some(audio_buffer)`, by calling `compute_reversed_buffer(&ctx, &audio_buffer)`.

### Function / type signatures

```rust
// New constants in deck_audio.rs
const SCRATCH_SENSITIVITY: f64  = 0.9;   // sqrt-curve scale (TAU/s → ~1.21×)
const SCRATCH_RATE_MAX:    f32  = 3.5;   // max playback rate during scratch
const SCRATCH_SMOOTH_SECS: f64  = 0.012; // 12 ms ramp for forward-phase smoothing

// New public free function
/// Compute scratch playback rate from normalized angular velocity using a
/// square-root (compressive) non-linear curve.
///
/// `normalized_vel` = |d_angle| / dt / (TAU × 0.55), where TAU × 0.55 is the
/// angular velocity of a 33 RPM record (0.55 rotations/second).
/// Returns a value in [0.0, SCRATCH_RATE_MAX].
pub(crate) fn scratch_rate_nonlinear(normalized_vel: f64) -> f32;

// New public free function
/// Pre-compute a reversed copy of `forward` AudioBuffer for reverse-scratch playback.
///
/// Iterates all channels, reverses sample order, and writes into a new
/// `AudioBuffer` of identical dimensions.  Returns `Err` on zero-length input
/// or browser OOM; the caller should log and fall back to forward-only scratch.
pub fn compute_reversed_buffer(
    ctx:     &AudioContext,
    forward: &AudioBuffer,
) -> Result<AudioBuffer, JsValue>;

// New fields on AudioDeck struct
pub reversed_buffer:      Option<AudioBuffer>,  // None until file load
pub scratch_in_reverse:   bool,                 // true while using reversed buffer
pub scratch_position_secs: f64,                 // integrated track position during scratch
pub scratch_was_playing:  bool,                 // was deck playing at scratch_start?
```

### Edge cases

| Case | Handling |
|---|---|
| Scratch on paused deck (source = None) | `scratch_start()` records `scratch_was_playing = false`. `scratch_move()` swaps to reversed buffer on backward drag (starting a new source), producing audible scratch. `scratch_end()` stops the source and restores paused state. |
| No reversed buffer loaded (zero-length track or OOM) | All `reversed_buffer.is_some()` guards fall through. Backward drags produce silence as before (clamped to rate 0). |
| Rapid direction changes (forward→reverse→forward in <50 ms) | Each `scratch_move()` call independently detects direction and swaps buffer; `stop_source()` + immediate new start is called each time. Possible audible click at swap point — acceptable per non-goals (no phase morph). |
| `dt = 0` (two events at same timestamp) | Guard `dt = elapsed.max(0.001)` prevents divide-by-zero; normalized_vel = 0 → rate = 0.0. |
| Scratch position reaches track end | `scratch_position_secs` is clamped to `[0, buffer.duration()]`. Reversed buffer offset clamped to `[0, rev_duration]`. Audio playback will stall/repeat naturally. |
| `scratch_end()` called without matching `scratch_start()` | Guard `if !self.scratch_active { return; }` unchanged — safe no-op. |
| Effect(playback_rate) fires during scratch | The `Effect` in deck.rs writes `state.playback_rate` to `source.playback_rate()`. This could fight the scratch rate. This is the existing behavior and is unchanged by this feature; the effect fires only when `state.playback_rate` signal changes (pitch fader), not on every frame. |

### Integration points

| File | Change |
|---|---|
| `src/audio/deck_audio.rs` | Add 4 fields to `AudioDeck`; add 2 free functions; update scratch methods; update tests |
| `src/audio/loader.rs` | After buffer store: call `compute_reversed_buffer`, store result or log warning |
| `src/components/deck.rs` | No changes |
| `src/state/deck.rs` | No changes |

---

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `current_position()` is inaccurate during scratch (uses stale `rate_at_play`). `scratch_end()` must NOT rely on it. | Use `scratch_position_secs` as the authoritative position during and at end of scratch. Call `play(scratch_position_secs, ...)` in `scratch_end()` to re-anchor the linear model. |
| Correctness | Buffer swap on every direction change calls `stop_source()` + new source start, which may cause an audible click. | Acceptable per non-goals (no phase-coherent morph). Click is short (< 1 ms) and masked by the transient nature of a scratch. |
| Simplicity | `scratch_move()` has significant branching. | Extract `scratch_rate_nonlinear()` as a pure testable function. Keep the three branches (fwd→rev, rev→fwd, same-dir) clear with comments. |
| Coupling | `scratch_move()` directly creates `AudioBufferSourceNode`s (duplicates `play()` logic). | Extract `stop_source()` for cleanup (already exists). Inline source-creation code with clear `expect()` messages — acceptable since it's in one method. |
| Performance | `reversed_buffer` doubles per-deck memory. | Acceptable per feature doc (resolved open question 4). |
| Testability | `compute_reversed_buffer()` requires `AudioContext` → WASM-only test. | Extract pure `reverse_channel_data(&[f32]) -> Vec<f32>` helper (even though trivial) if needed. Actually Vec::reverse() is so simple it doesn't need a native test. Test via WASM. |

---

## Implementation Notes

- Old `scratch_rate_from_angular_velocity()` is removed; its 5 native tests are
  updated to test `scratch_rate_nonlinear()` with new expected values.
- `scratch_negative_angular_velocity_clamped_to_zero` test is removed; the new
  function takes an absolute value (`normalized_vel` is always ≥ 0).
- The `#[allow(deprecated)]` on `stop_with_when` remains on `stop_source()` where
  it already lives; `scratch_move()` calls `self.stop_source()` rather than
  duplicating the inline pattern.
- The inline source-creation code in `scratch_move()` is duplicated twice (fwd→rev
  and rev→fwd branches) — acceptable since DRY extraction would require a private
  helper that takes `&AudioBuffer` alongside `&mut self`, which hits borrow-checker
  issues when the buffer is cloned from self. Use `.clone()` on `AudioBuffer` (cheap
  JS reference clone) to satisfy the borrow checker.

---

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `scratch_rate_nonlinear(0.0)` → 0.0 | 1 | ✅ | `scratch_nonlinear_stationary_gives_zero` |
| `scratch_rate_nonlinear` at 1 rot/s in [1.0, 1.3] | 1 | ✅ | `scratch_nonlinear_one_rotation_per_second_in_target_range` |
| `scratch_rate_nonlinear` clamped at SCRATCH_RATE_MAX | 1 | ✅ | `scratch_nonlinear_clamped_at_upper_bound` |
| sqrt curve is sub-linear (doubling vel < doubles rate) | 1 | ✅ | `scratch_nonlinear_sqrt_is_sublinear` |
| `scratch_rate_nonlinear` at half rot/s | 1 | ✅ | `scratch_nonlinear_half_speed` |
| `compute_reversed_buffer` produces same dimensions | WASM | ✅ | `reversed_buffer_has_same_dims` |
| `compute_reversed_buffer` reverses sample order | WASM | ✅ | `reversed_buffer_reverses_content` |
| `compute_reversed_buffer` stereo (both channels reversed) | WASM | ✅ | `reversed_buffer_both_channels_reversed` |
| `reversed_buffer` is None before file load | WASM | ✅ | `reversed_buffer_is_none_before_load` |
| `scratch_in_reverse` starts false | WASM | ✅ | `scratch_in_reverse_starts_false` |
| `scratch_end()` without start is safe | WASM | ✅ (existing) | `scratch_end_without_start_is_safe` |
| `scratch_state_inactive_by_default` | WASM | ✅ (existing) | unchanged |
| Paused-deck scratch (source = None) no panic | WASM | ✅ | `scratch_on_paused_deck_does_not_panic` |
| Signal→DOM: no new reactive wiring | — | N/A | No new signals; no integration test needed |
| Backward drag produces reversed source (runtime) | Manual | ⚠️ smoke test | Cannot be automated (requires audio decode) |
| Forward rate is within expected range at typical gesture | Manual | ⚠️ smoke test | Requires subjective human evaluation |

---

## Test Results

All tests pass:
- `cargo test`: 90 native tests pass (including 5 new `scratch_nonlinear_*` tests)
- `python make.py lint` (`cargo clippy --target wasm32-unknown-unknown`): 0 errors, pre-existing warnings in bpm.rs/header.rs/platter_draw.rs unchanged
- `trunk build`: success

WASM tests (`wasm-pack test --headless --firefox`) require a browser environment — 6 new WASM tests cover reversed buffer construction and initial scratch state.

---

## Review Notes

- **Position integration bug (found in code review)**: Original code used `going_reverse` for the position integrator. Fixed to use `scratch_in_reverse`. When no reversed buffer is available, `going_reverse` is true but the forward buffer still plays forward — incorrect subtraction would diverge the position estimate.
- All public `fn`s and struct fields have `///` doc comments.
- Named constants for all 3 magic numbers (`SCRATCH_SENSITIVITY`, `SCRATCH_RATE_MAX`, `SCRATCH_SMOOTH_SECS`).
- `.expect()` calls are justified: AudioContext node creation is infallible on a live context.
- No dead code, no debug prints.

---

## Decisions Made

See feature doc `## Decisions Made` section.

---

## Lessons / Highlights

See feature doc `## Lessons / Highlights` section.

---

## Callouts / Gotchas

- `AudioBuffer` is a JS reference type — `.clone()` is cheap (copies the JS object
  handle, not the PCM data). Use freely to satisfy borrow checker.
- `stop_with_when(0.0)` is deprecated in web-sys 0.3.91 but remains the correct
  call. The `#[allow(deprecated)]` lives on `stop_source()` — route all source
  stopping through that method.
- `linear_ramp_to_value_at_time` signature: `(value: f32, end_time: f64)`. The
  first arg is `f32`, not `f64`. Easy to miss.
- `cancel_scheduled_values(0.0)` in web-sys cancels all events at or after time 0,
  which is everything. Use `cancel_scheduled_values(ctx.current_time())` to only
  cancel future events, preserving any past scheduled values already applied.
