# M4 — Mirror Symmetry Core

**Status:** ✅ Complete  
**Depends on:** [M3 — Image Input & Texture Display](m3-image-input.md)  
**Unlocks:** M5, M6, M7, M8, M9

---

## Goal

This is the milestone where the app becomes a kaleidoscope. Implement the polar
coordinate transform and mirror fold in the fragment shader, wire up the four
core controls (segments, rotation, zoom, center drag), and connect Leptos signals
to WebGL uniforms via the signal→render `Effect`. After this milestone the app
is demonstrably usable as a kaleidoscope tool.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Flesh out `src/state/params.rs` with full `KaleidoscopeParams` struct (all `RwSignal`s per tech spec Section 5.1) and `fn new()` + `impl Default` | ✅ |
| 2 | Provide `KaleidoscopeParams` via Leptos context from `App`; update `CanvasView` to read it with `expect_context` | ✅ |
| 3 | Add `ParamsSnapshot` plain struct (no signals) that `KaleidoscopeParams::snapshot()` populates by reading all signals — this is what gets passed to `renderer.draw()` | ✅ |
| 4 | Update `renderer/uniforms.rs` to cache and upload all symmetry uniforms: `u_segments`, `u_rotation`, `u_zoom`, `u_center` | ✅ |
| 5 | Update `frag.glsl` — implement polar coordinate transform centred on `u_center` | ✅ |
| 6 | Update `frag.glsl` — implement mirror fold: `seg_angle = PI / segments`; `a = mod(a, 2*seg_angle)`; fold back if `a > seg_angle` | ✅ |
| 7 | Update `frag.glsl` — apply `u_rotation` offset to angle, `u_zoom` scale to radius before texture sample | ✅ |
| 8 | Update the `Effect` in `CanvasView` to call `params.snapshot()` (registering all signals as deps) and pass snapshot to `renderer.draw()` | ✅ |
| 9 | Create `src/components/controls_panel.rs` — panel wrapper component (not yet collapsible); renders four controls | ✅ |
| 10 | Add segments slider (integer, 2–10) in controls panel; wires to `params.segments` | ✅ |
| 11 | Add rotation slider (0–360°) in controls panel; wires to `params.rotation` | ✅ |
| 12 | Add zoom slider (0.1–4.0) in controls panel; wires to `params.zoom` | ✅ |
| 13 | Implement canvas center drag — attach `pointerdown`, `pointermove`, `pointerup` listeners on `<canvas>`; normalise pointer position to 0..1 and write to `params.center` | ✅ |
| 14 | Extract mirror fold math as a pure Rust function in `utils.rs`; write `#[test]` unit tests for edge cases (a=0, a=seg_angle, a=2*seg_angle, segments=2, segments=10) | ✅ |
| 15 | Verify `python make.py build`, `python make.py lint`, and `python make.py test` all pass | ✅ |

---

## Manual Test Checklist

- [ ] Upload any image → kaleidoscope pattern visible (not just a passthrough)
- [ ] Drag segments slider from 2 to 10 → pattern updates each step in real time
- [ ] 2 segments produces an infinite-corridor / mirror-tunnel look
- [ ] 3 segments produces a classic mandala
- [ ] 6 segments produces a hexagonal snowflake
- [ ] Drag rotation slider → pattern rotates smoothly
- [ ] Drag zoom slider → source image zooms in/out
- [ ] Click and drag directly on canvas → center of symmetry moves, pattern updates
- [ ] No console errors

---

## Notes

- `params.snapshot()` must read every signal to register all of them as reactive
  dependencies of the `Effect`. Missing a signal means that control won't trigger
  a redraw.
- The `PointerEvent` listeners on `<canvas>` must call `event.prevent_default()`
  to block browser text selection / scroll while dragging.
- Keep pointer listener `EventListener` handles alive with `std::mem::forget`
  (see `rust-wasm-debug` skill).
- Unit-test the fold math in native Rust before testing in the browser — it's
  easy to get the modular arithmetic wrong.
