# Bug 03 — Corrupted Duration Display in Audio Player

## Status
Fixed

---

## Summary

When viewing a recording, the duration display in the audio player shows a corrupted value such as `0:05307445734561825860:15` instead of something like `0:05 / 0:05`. The number is nonsensically large and renders the playback UI unreadable.

---

## Observed Behaviour

Duration shown in player: `0:05307445734561825860:15`

The current time (`0:05`) and the corrupted duration (`307445734561825860:15`) are rendered adjacent in the UI, producing the fused string above.

---

## Possible Issue (Initial Assessment)

`HTMLMediaElement.duration` can return `Infinity` or `NaN` before audio metadata is fully resolved. If `format_duration` receives a non-finite value, casting it to `u64` produces an enormous or undefined integer:

- `f64::INFINITY as u64` → `u64::MAX` = 18,446,744,073,709,551,615
- `u64::MAX / 60` → 307,445,734,561,825,860 (the minute component in the corrupted string)

---

## Root Cause

`format_duration` in `src/ui/audio_player.rs` had no guard for non-finite inputs:

```rust
// Before — no finite check
pub fn format_duration(secs: f64) -> String {
    let total = secs as u64;   // ← undefined/saturating cast if secs is Inf or NaN
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}
```

In Rust, casting `f64::INFINITY as u64` saturates to `u64::MAX`. The `onloadedmetadata` handler sets `duration` from `a.duration()`, which the browser briefly returns as `Infinity` for some audio formats before metadata is fully parsed. This passes directly into `format_duration`, producing the garbage output.

---

## Fix

Added an early return for non-finite and negative values in `format_duration`:

```rust
pub fn format_duration(secs: f64) -> String {
    if !secs.is_finite() || secs < 0.0 {
        return "0:00".to_string();
    }
    let total = secs as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}
```

This is safe because the `loaded` signal in the component already gates the duration display — `"0:00"` is shown until `loadedmetadata` fires with a valid finite value, at which point the duration renders correctly.

---

## Tests Added

Three new unit tests in `src/ui/audio_player.rs`:

| Test | Input | Expected |
|------|-------|----------|
| `format_infinity_returns_zero` | `f64::INFINITY` | `"0:00"` |
| `format_nan_returns_zero` | `f64::NAN` | `"0:00"` |
| `format_negative_returns_zero` | `-1.0` | `"0:00"` |

All 31 tests pass.

---

## Lessons Learned

### 1. `f64 as u64` saturates on non-finite values — silently

In Rust, casting a non-finite `f64` to an integer type is a saturating cast: `f64::INFINITY as u64` = `u64::MAX`, `f64::NAN as u64` = `0` (platform-dependent in older Rust; defined as `0` since Rust 1.45). Neither panics. The bug can sit undetected until a real audio file triggers the edge case.

**Rule:** Any function that accepts a `f64` representing a real-world measurement (time, distance, size) should validate with `.is_finite()` before casting to an integer type.

### 2. Browser API values are not always valid on first delivery

`HTMLMediaElement.duration` is `Infinity` for live streams and transiently `NaN` during loading for some formats. Defensive handling at the formatting layer is more robust than relying on the browser to always deliver a tidy value — the component's `loaded` guard helps, but a race or format quirk can still deliver `Infinity` to `format_duration`.

