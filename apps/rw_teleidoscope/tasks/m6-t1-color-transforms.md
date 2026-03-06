# Task M6-T1: Color Transforms

**Milestone:** M6 — Color Transforms  
**Status:** 🔄 In Progress

## Restatement

Add five post-processing color transforms (hue rotation, saturation, brightness,
posterize, invert) to the fragment shader as uniform-driven operations applied
after the kaleidoscope texture sample. Wire each uniform to a new control in the
side panel. `KaleidoscopeParams` and `ParamsSnapshot` already contain all five
fields from an earlier session; only the shader GLSL, `UniformLocations`,
`controls_panel.rs`, and `utils.rs` need updating. No server-side or network
changes are required. Out of scope: camera input, export, M9 randomise.

## Design

### Data flow

User moves a slider / toggles a checkbox →
`on:input` / `on:change` handler writes to the appropriate `RwSignal<_>` in
`KaleidoscopeParams` → Leptos reactive `Effect` in `CanvasView` re-calls
`params.snapshot()` → `renderer.draw(&snapshot)` → `uniforms.upload(gl, &snapshot)`
uploads the five new uniforms → fragment shader applies color transforms.

### Function / type signatures

**`src/utils.rs`** (new, pure Rust, native-testable):

```rust
/// Pure Rust equivalent of the GLSL RGB→HSV→RGB hue-rotation.
/// Returns the (r, g, b) triple after rotating the hue by `degrees`.
pub fn hue_rotate_rgb(r: f32, g: f32, b: f32, degrees: f32) -> (f32, f32, f32)

/// Pure Rust equivalent of the GLSL posterize formula.
/// Clamps `v` to [0,1], quantises to `levels` bands.
/// Returns `floor(v * levels) / levels`.
pub fn posterize_channel(v: f32, levels: u32) -> f32
```

**`src/renderer/uniforms.rs`** (additions to `UniformLocations`):
```rust
pub u_hue_shift:  Option<glow::UniformLocation>,
pub u_saturation: Option<glow::UniformLocation>,
pub u_brightness: Option<glow::UniformLocation>,
pub u_posterize:  Option<glow::UniformLocation>,
pub u_invert:     Option<glow::UniformLocation>,
```

**`assets/shaders/frag.glsl`** (new GLSL helpers + uniform declarations):
```glsl
uniform float u_hue_shift;   // 0–360 degrees
uniform float u_saturation;  // 0–2; 1=unchanged
uniform float u_brightness;  // 0–2; 1=unchanged
uniform int   u_posterize;   // 0=off, 2–16=levels
uniform int   u_invert;      // 0=off, 1=on
```

### Edge cases

- `u_hue_shift == 0.0`: no-op; shader skips rotation (short-circuit guard).
- `u_saturation == 1.0`: mix(lum, colour, 1.0) = colour — identity.
- `u_posterize == 0`: guarded with `if (u_posterize > 1)` as spec says.
- `u_posterize == 1`: only 1 level = flat black; treat same as > 1 for now (spec says 2–16 from UI but shader must not divide by zero when 1 is passed; `floor(v*1)/1 = 0` which is valid).
- Colour values outside [0,1] after brightness multiply: posterize clamps to [0,1] first.
- `degrees = 360.0` rounds back to 0 via `mod(..., 360.0)`.

### Integration points

| File | Change |
|---|---|
| `assets/shaders/frag.glsl` | Add 5 uniform declarations + 3 GLSL helpers + apply in main() |
| `src/renderer/uniforms.rs` | Add 5 `UniformLocations` fields + `new()` queries + `upload()` |
| `src/components/controls_panel.rs` | Add Color section with 5 controls |
| `src/utils.rs` | Add `hue_rotate_rgb`, `posterize_channel` + unit tests |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `mod(hue, 6.0)` branch table in `hsv_to_rgb` must handle exactly i=5 | Explicit `else` for i≥5 (range [5,6)) covers it |
| Correctness | `rgb_to_hsv` uses `==` float comparisons on `cmax` | Standard GLSL practice; cmax was computed from the same vars so the equality is exact |
| Simplicity | HSV round-trip is verbose | Acceptable — keeps hue math correct |
| Coupling | Color order (hue→sat→brightness→posterize→invert) is spec-mandated | Follow m6 doc order exactly |
| Performance | 5 extra uniforms per draw call | Negligible — uniform upload is near-zero cost |
| Testability | GLSL functions can't be unit-tested directly | Mirror in pure Rust in `utils.rs` and test there |

## Implementation Notes

- Colour transform application order per m6 doc: hue → sat → brightness → posterize → invert.
- `saturate_rgb` uses Rec. 709 luminance coefficients: `vec3(0.2126, 0.7152, 0.0722)`.
- `u_invert` is uploaded as `i32` (like `u_mobius`) since GLSL `bool` uniform → `gl.uniform_1_i32`.

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `hue_rotate_rgb` at 0° → identity | 1 | ✅ | utils.rs unit test |
| `hue_rotate_rgb` at 180° → hue shifted | 1 | ✅ | utils.rs unit test |
| `hue_rotate_rgb` at 360° → wraps to identity | 1 | ✅ | utils.rs unit test |
| `posterize_channel` at levels=2 → banded | 1 | ✅ | utils.rs unit test |
| `posterize_channel` at levels=16 → subtle | 1 | ✅ | utils.rs unit test |
| `posterize_channel` clamps values > 1 | 1 | ✅ | utils.rs unit test |
| Shader uniform upload (5 new uniforms) | 2 | ❌ waived | requires WebGL context; covered by MT checklist |
| Slider → signal → DOM display reactive | 3 | ❌ deferred | manual test checklist MT items cover visual behaviour |

## Test Results

(filled after Phase 6)

## Review Notes

(filled after Phase 7)

## Callouts / Gotchas

(filled after Phase 10)
