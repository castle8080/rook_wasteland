# Task M5: Visual Effects

**Milestone:** M5 — Visual Effects  
**Status:** 🔄 In Progress

## Restatement

This task adds six visual-effect transforms to the fragment shader and wires each
to UI controls in the controls panel.  Five effects (spiral, ripple, lens, radial
fold, and Möbius flip) are implemented as new GLSL uniforms applied in the
pipeline between polar-coordinate computation and texture sampling.  A sixth
effect—recursive reflection—renders the kaleidoscope output into a framebuffer
object (FBO) and feeds that result back as the source texture for up to three
additional passes.  The colour-transform uniforms (hue, saturation, etc.) are out
of scope for M5; those belong to M6.  The `KaleidoscopeParams` and
`ParamsSnapshot` structs already contain all the necessary signal fields from M4
planning work; no state changes are needed.

## Design

### Data flow

User moves a slider → `RwSignal` in `KaleidoscopeParams` fires → `params.snapshot()`
in the Leptos `Effect` in `CanvasView` captures the new value → `renderer.draw(&snap)`
is called → `uniforms.upload()` pushes the value to the GPU → fragment shader
reads it and alters the sample UV.

For recursive reflection: `snap.recursive_depth > 0` → `Renderer::draw()` enters
the multi-pass loop → first pass renders source → FBO texture A → subsequent
passes render FBO A → FBO B (ping-pong) → final pass renders last FBO → default
framebuffer.

### Function / type signatures

```rust
// utils.rs — pure Rust equivalents for unit testing
/// GLSL barrel-warp formula: r / max(1 - lens*r*r, 0.001).
pub fn lens_warp(r: f32, lens: f32) -> f32;

/// GLSL radial-fold formula: abs(mod(r*(1+fold*4), 2) - 1).
pub fn radial_fold_r(r: f32, fold: f32) -> f32;

// renderer/mod.rs — Renderer gains two new fields
pub struct Renderer {
    // ... existing fields ...
    fbo: glow::Framebuffer,
    fbo_textures: [glow::Texture; 2],
}
```

### Edge cases

- `u_lens` denominator clamped to `max(…, 0.001)` to prevent division by zero.
- `u_radial_fold == 0.0` → skip formula entirely (not identity when applied).
- Möbius `seg_idx` derived from pre-fold angle; GLSL `mod(x, y)` with y>0 is
  always ≥ 0, so negative angles are handled correctly.
- `recursive_depth == 0` → skip all FBO passes; render directly to canvas.
- FBO ping-pong with 2 textures avoids reading and writing the same texture.

### Integration points

| File | Change |
|---|---|
| `assets/shaders/frag.glsl` | Add 5 uniforms + effects pipeline |
| `src/renderer/uniforms.rs` | Add 5 location fields + upload calls |
| `src/renderer/mod.rs` | Add `fbo` + `fbo_textures`; multi-pass draw; Drop |
| `src/components/controls_panel.rs` | Add sliders + Möbius toggle + depth slider |
| `src/utils.rs` | Add `lens_warp` + `radial_fold_r` + unit tests |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | FBO read/write same texture not allowed in WebGL | Use two FBO textures and ping-pong between them |
| Simplicity | Multi-pass draw adds branching complexity to `Renderer::draw` | Encapsulate in a single `draw_multi_pass` helper block inside `draw` |
| Coupling | `draw.rs::draw_frame` signature already accepts `Option<Texture>` | No signature change needed; FBO texture is passed as `source_texture` arg |
| Performance | FBO alloc per-frame would be expensive | FBO and textures allocated once in `Renderer::new()`; reused every frame |
| Testability | GLSL formulas only testable in browser | Pure Rust mirrors added to `utils.rs` and tested with `cargo test` |

## Implementation Notes

- Möbius uses `r = -r` on odd segments. With `fract()` in the sample UV,
  negative r wraps around the texture creating the alternating-flip effect.
- `seg_idx = floor(a / two_seg)` must be computed *before* the `mod()` fold.
- Radial fold is gated by `if (u_radial_fold > 0.0)` in the shader because the
  formula is not an identity at fold=0.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `lens_warp(0, any)` → 0 | 1 | ✅ | utils.rs unit test |
| `lens_warp(r, 0)` → identity | 1 | ✅ | utils.rs unit test |
| `lens_warp` denominator clamp | 1 | ✅ | utils.rs unit test |
| `radial_fold_r` happy path | 1 | ✅ | utils.rs unit test |
| `radial_fold_r` at fold boundary | 1 | ✅ | utils.rs unit test |
| Shader effects compile without error | 2 | ✅ | wasm-pack integration test (Renderer::new succeeds) |
| Slider → signal → redraw | 3 | ❌ waived | Covered by manual test checklist; adding a DOM integration test for every new slider is disproportionate given M4 already validates the signal→redraw wiring |
| Recursive depth 0 → normal render | 2 | ✅ | covered by existing integration tests that mount App |

## Test Results

(filled after running `python make.py test`)

## Review Notes

(filled after self-review)

## Callouts / Gotchas
