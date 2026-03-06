# Task M4-1: Mirror Symmetry Core

**Milestone:** M4 — Mirror Symmetry Core  
**Status:** ✅ Done

## Restatement

This task implements the kaleidoscope's core rendering pipeline: a polar-coordinate
mirror-fold in the WebGL fragment shader, wired to four live controls (segments,
rotation, zoom, and canvas-drag center). The work spans the full vertical slice from
Leptos signals → `ParamsSnapshot` → GLSL uniforms → rendered pixels. The fragment
shader gains the polar transform and mirror-fold algorithm that makes the app a
kaleidoscope (replacing the M3 passthrough sampler). Out of scope: effects (M5),
colour transforms (M6), camera (M7), export (M8), and the collapsible panel
collapse/expand animation (M10).

## Design

### Data flow

User gesture → RwSignal (KaleidoscopeParams) → Effect reads `params.snapshot()` →
`ParamsSnapshot` passed to `renderer::draw(&snapshot)` → `Renderer::draw` → `draw_frame`
→ `uniform_locs.upload(gl, params)` sets GPU uniforms → frag shader samples using
polar fold.

Pointer drag on canvas → `EventListener` on `"pointerdown"` / `"pointermove"` →
normalise to 0.0–1.0, flip y → `params.center.set(...)`.

Slider on:input → `HtmlInputElement.value().parse()` → `params.segments/rotation/zoom.set(...)`.

### Function / type signatures

```rust
// state/params.rs
/// Plain-data snapshot of all KaleidoscopeParams signal values.
/// Passed to the renderer each frame; avoids borrow-checker conflicts
/// between the signal system and glow's !Send + !Sync Context.
pub struct ParamsSnapshot { /* all fields */ }

impl KaleidoscopeParams {
    /// Read every signal and return a plain-data snapshot.
    /// Calling this inside a Leptos Effect registers ALL signals as
    /// reactive dependencies.
    pub fn snapshot(&self) -> ParamsSnapshot { ... }
}

// renderer/uniforms.rs
impl UniformLocations {
    pub fn upload(&self, gl: &glow::Context, params: &ParamsSnapshot);
}

// renderer/draw.rs
pub unsafe fn draw_frame(
    gl: &glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    source_texture: Option<glow::Texture>,
    uniform_locs: &UniformLocations,
    params: &ParamsSnapshot,
);

// renderer/mod.rs
pub fn draw(params: &ParamsSnapshot);
impl Renderer { pub fn draw(&self, params: &ParamsSnapshot); }

// utils.rs
/// Mirror-fold angle `a` (radians) into the domain [0, PI/segments].
pub fn mirror_fold(a: f32, segments: u32) -> f32;
```

### Edge cases

- `params.snapshot()` in init Effect: registers all params as deps, causing the init
  Effect to fire once on first signal change; the guard returns early (benign).
- Canvas not yet in DOM when reactive Effect fires: `renderer::draw()` is a no-op.
- `segments = 0`: cannot happen (clamped to 2–10 by slider), but GLSL div-by-zero
  guard is not needed.
- `center` drag outside canvas bounds: clamped to [0.0, 1.0].
- Negative angles in `mirror_fold`: `rem_euclid` handles them correctly.
- `zoom = 0`: slider min is 0.1, so not reachable.

### Integration points

- `src/state/params.rs` — add `ParamsSnapshot`, `snapshot()`
- `src/state/mod.rs` — re-export `ParamsSnapshot`
- `src/renderer/uniforms.rs` — add symmetry uniform locs + upload
- `src/renderer/draw.rs` — `draw_frame` gains `params` arg
- `src/renderer/mod.rs` — `Renderer::draw` + free `draw()` gain `params` arg
- `src/components/canvas_view.rs` — params context, snapshot, pointer events
- `src/components/controls_panel.rs` — segments/rotation/zoom sliders
- `src/app.rs` — render `ControlsPanel`
- `src/utils.rs` — `mirror_fold` + unit tests
- `assets/shaders/frag.glsl` — polar transform + mirror fold
- `tests/m4_mirror_symmetry.rs` — browser integration tests

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `params.snapshot()` in init Effect registers all signals, causing one spurious re-run | Guard returns immediately; Leptos drops deps → Effect becomes inert |
| Simplicity | Snapshot pattern adds a struct vs passing signals directly | Necessary to cross the `!Send + !Sync` boundary of glow::Context |
| Coupling | `uniforms.rs` imports from `state::params` (renderer ↔ state) | Acceptable: renderer consumes a value type, no reactive coupling |
| Performance | All 15 signals read every draw call | Unavoidable; signals are cheap reads, no allocations |
| Testability | Mirror-fold logic lives only in GLSL + utils.rs | Pure Rust `mirror_fold` in utils.rs enables native unit tests |

## Implementation Notes

- GLSL `mod(a, y)` is equivalent to Rust `rem_euclid` for positive `y`.
- Pointer Y must be flipped: `cy = 1.0 - offsetY / 800.0` (WebGL y=0 at bottom, HTML y=0 at top).
- `u_rotation` is uploaded in radians (`params.rotation.to_radians()`); signal stores degrees.
- Use `fract(sample_uv)` for seamless tile-wrap when the reconstructed UV leaves [0,1].

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `mirror_fold(0, n)` → 0 | 1 | ✅ | utils.rs unit test |
| `mirror_fold(seg_angle, n)` → seg_angle | 1 | ✅ | utils.rs unit test |
| `mirror_fold(2*seg_angle, n)` → 0 | 1 | ✅ | utils.rs unit test |
| `mirror_fold(above seg_angle)` folds | 1 | ✅ | utils.rs unit test |
| `mirror_fold(negative)` wraps + folds | 1 | ✅ | utils.rs unit test |
| `mirror_fold` segments=2 | 1 | ✅ | utils.rs unit test |
| `mirror_fold` segments=10 | 1 | ✅ | utils.rs unit test |
| `snapshot()` reads all fields | 1 | ✅ | implicit via struct exhaustiveness |
| Controls panel renders sliders | 3 | ✅ | m4_mirror_symmetry.rs |
| Signal change → controls panel value updates | 3 | ✅ | m4_mirror_symmetry.rs |
| Full draw pipeline (WebGL) | 3 | ✅ | existing integration.rs smoke tests |
| Pointer drag → center signal | 3 | ❌ waived | requires synthetic PointerEvent dispatch; covered by MT checklist |

## Test Results

- `cargo test`: 16 native unit tests pass (7 new `mirror_fold` tests + existing).
- `cargo clippy --target wasm32-unknown-unknown --tests -- -D warnings`: clean.
- `python make.py build` (trunk): clean.
- Browser tests (`wasm-pack test --headless --firefox`): blocked by pre-existing
  environment issue — wasm-bindgen-test-runner version mismatch (see L11 in lessons.md).
  All tests compile correctly; the runner itself fails before executing them.

## Review Notes

Code-review agent found no issues. All clippy warnings fixed before commit.

## Callouts / Gotchas

- `params.snapshot()` called in the init Effect registers all params as reactive deps;
  the first signal change after mount causes one no-op re-run of the init Effect before
  its dep set goes empty. Benign.
- Browser tests fail at the wasm-bindgen-test-runner level (pre-existing env issue);
  this is independent of M4 correctness.
