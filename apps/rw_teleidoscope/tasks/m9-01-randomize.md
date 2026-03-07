# Task M9-01: Randomize ("Surprise Me")

**Milestone:** M9 — Randomize  
**Status:** 🔄 In Progress

## Restatement

This task implements the "Surprise Me" feature: a single public function
`randomize(params: KaleidoscopeParams)` that sets all kaleidoscope signals to
randomly chosen values within visually safe ranges. It lives in
`src/state/randomize.rs`, gated with `#[cfg(target_arch = "wasm32")]` because
it uses `js_sys::Math::random()`. A "⚡ SURPRISE ME" button is added to
`controls_panel.rs` directly above `<ExportMenu/>` (per wireframe), wired to
call `randomize(params)` on click and disabled when no image is loaded.
Posterize and invert are deliberately kept off during randomization to avoid
visually broken output.

## Design

### Data flow

User clicks "SURPRISE ME" button → `on:click` handler calls
`state::randomize(params)` → `randomize` reads `js_sys::Math::random()` and
calls `.set()` on every `KaleidoscopeParams` signal → reactive effects in
`CanvasView` trigger a redraw → slider DOM elements re-render reactively from
signal values.

### Function / type signatures

```rust
// src/state/randomize.rs
#[cfg(target_arch = "wasm32")]

/// Randomise all [`KaleidoscopeParams`] signals to visually interesting values.
///
/// Parameter ranges:
/// - `segments`: integer 2–10
/// - `rotation`: f32 0.0–360.0°
/// - `center`: (f32, f32) both in 0.2–0.8 (avoids extreme edges)
/// - `zoom`: f32 0.3–2.5
/// - effects: 1 or 2 of {spiral, radial_fold, lens, ripple} at intensity 0.2–1.0;
///   the rest set to 0.0
/// - `mobius`: true with 30% probability
/// - `recursive_depth`: 0 (60% chance), 1 (20%), 2 (20%)
/// - `hue_shift`: f32 0.0–360.0
/// - `saturation`: f32 0.8–1.8
/// - `brightness`: f32 0.7–1.5
/// - `posterize`: always 0 (off)
/// - `invert`: always false
pub fn randomize(params: KaleidoscopeParams) { ... }
```

### Edge cases

- `js_sys::Math::random()` returns `[0.0, 1.0)` — upper bound is exclusive.
  Multiplying `random() * N` then truncating to `usize` is safe for small N.
- Segment count calculation: `2 + (random() * 9.0) as u32` yields 2..=10
  (since 9.0 * (1.0 - ε) < 9).
- Fisher-Yates partial shuffle for effect selection: loop i in 0..count,
  swap `indices[i]` with `indices[i + random_offset]`. `count` is at most 2,
  so i is at most 1, and `4 - i` is at least 2 — no division by zero.

### Integration points

- **New:** `src/state/randomize.rs` — the function
- **Modified:** `src/state/mod.rs` — add `#[cfg(target_arch="wasm32")] mod randomize; pub use randomize::randomize;`
- **Modified:** `src/components/controls_panel.rs` — add Surprise Me button
- **New:** `tests/m9_randomize.rs` — browser tests

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Fisher-Yates offset could index past array end | `i + (random() * (4-i) as f64) as usize` ≤ `i + (4-i-1)` = 3; always in bounds |
| Simplicity | Could use a seeded PRNG for testability | Spec mandates `js_sys::Math::random()`; tests check bounds, not specific values |
| Coupling | `randomize` sets 15 signals — tightly coupled to `KaleidoscopeParams` shape | Acceptable; it's a convenience mutation function whose entire purpose is to touch all params |
| Performance | 15 signal `.set()` calls could trigger 15 reactive redraws | Leptos 0.8 batches reactive updates within the same synchronous call, so only one redraw occurs |
| Testability | `js_sys::Math::random()` is non-deterministic — tests cannot assert exact values | Test invariants (bounds, always-off fields) rather than specific values |

## Implementation Notes

- `KaleidoscopeParams` is `Copy` in Leptos 0.8 (all fields are `RwSignal<T>` which are
  `Copy`), so the function can take it by value without a reference.
- Button is disabled when `!app_state.image_loaded.get()` to match the wireframe
  note that controls are non-interactive until an image is loaded.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| segments in 2..=10 after randomize | 2 | ✅ | m9_randomize.rs |
| rotation in 0.0..=360.0 | 2 | ✅ | |
| center x and y in 0.2..0.8 | 2 | ✅ | |
| zoom in 0.3..=2.5 | 2 | ✅ | |
| at least 1 effect > 0 after randomize | 2 | ✅ | |
| at most 2 effects > 0 after randomize | 2 | ✅ | |
| active effects have intensity 0.2–1.0 | 2 | ✅ | |
| posterize always 0 | 2 | ✅ | |
| invert always false | 2 | ✅ | |
| hue_shift in 0.0..=360.0 | 2 | ✅ | |
| saturation in 0.8..=1.8 | 2 | ✅ | |
| brightness in 0.7..=1.5 | 2 | ✅ | |
| recursive_depth in 0..=2 | 2 | ✅ | |
| Surprise Me button present in DOM | 3 | ✅ | integration.rs |
| Button disabled when no image loaded | 3 | ✅ | integration.rs |

## Test Results

_(filled after Phase 6)_

## Review Notes

_(filled after Phase 7)_

## Callouts / Gotchas

_(filled after Phase 10)_
