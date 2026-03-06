# Task M2: WebGL Canvas & Basic Renderer

**Milestone:** M2 — WebGL Canvas & Basic Renderer  
**Status:** ✅ Done

## Restatement

Build the end-to-end WebGL 2 pipeline that proves rendering works: obtain a
`glow::Context` from the Leptos-managed `<canvas>` element, fetch the GLSL
shader files from Trunk's asset directory at runtime, compile and link them
into a `glow::Program`, upload a static full-screen quad geometry once, and
draw it to produce a solid steampunk-brass colour on the canvas.  

This lives in `src/renderer/` (context, shader, draw, mod) and
`src/components/canvas_view.rs`. No image texture, no uniforms, and no
animation loop are in scope — those begin in M3 and M4. The goal is solely to
confirm that WebGL initialises correctly and the shader compilation/fetch
pipeline functions end-to-end.

## Design

### Data flow

```
CanvasView mounts <canvas>
  → canvas_ref NodeRef populated
  → Effect fires
  → spawn_local(async { Renderer::new(&canvas) })
      → context::get_context(&canvas)        → glow::Context
      → shader::create_program(&gl)  (async) → glow::Program
          fetch "/rw_teleidoscope/assets/shaders/vert.glsl"
          fetch "/rw_teleidoscope/assets/shaders/frag.glsl"
          compile VERTEX_SHADER + FRAGMENT_SHADER
          link program
      → draw::create_quad(&gl)               → (glow::VertexArray, glow::Buffer)
  → renderer.draw()
      → gl.clear + gl.use_program + gl.bind_vertex_array + gl.draw_arrays
  → *renderer_ref.borrow_mut() = Some(renderer)
```

### Function / type signatures

```rust
// renderer/context.rs
/// Obtain a `glow::Context` from an `HtmlCanvasElement`.
pub fn get_context(canvas: &web_sys::HtmlCanvasElement) -> Result<glow::Context, String>

// renderer/shader.rs
/// Fetch vert + frag GLSL sources, compile, link into a program.
pub async fn create_program(gl: &glow::Context) -> Result<glow::Program, String>

// renderer/draw.rs
/// Allocate VAO + VBO for the full-screen quad.  Data uploaded once.
pub unsafe fn create_quad(
    gl: &glow::Context,
) -> Result<(glow::VertexArray, glow::Buffer), String>

/// Bind the program + VAO and issue the draw call.
pub unsafe fn draw_quad(
    gl: &glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
)

// renderer/mod.rs
pub struct Renderer { gl, program, vao, vbo }

impl Renderer {
    pub async fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, String>
    pub fn draw(&self)
}
```

### Edge cases

- `get_context("webgl2")` returns `None` → browser does not support WebGL 2 →
  surface error via `console.error`
- HTTP fetch returns non-200 → caught via `resp.ok()` check
- Shader compilation fails → GLSL info log surfaced via `console.error`
- Program link fails → link info log surfaced via `console.error`
- NodeRef fires with `None` on first reactive run → guarded with `if let Some`

### Integration points

- `src/renderer/context.rs` — new implementation
- `src/renderer/shader.rs` — new implementation
- `src/renderer/draw.rs` — new implementation
- `src/renderer/mod.rs` — new `Renderer` struct + `Drop` impl
- `src/components/canvas_view.rs` — replaces empty stub
- `src/app.rs` — adds `<CanvasView/>` to layout
- `assets/shaders/vert.glsl` — add `layout(location=0)`, keep passthrough UV
- `assets/shaders/frag.glsl` — change colour to brass `(0.545, 0.412, 0.078, 1.0)`
- `Cargo.toml` — add `"Response"`, `"console"` to web-sys features

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | Shader URL is hardcoded to `/rw_teleidoscope/` deployment path | Matches `Trunk.toml public_url`; acceptable until multi-deployment support is needed |
| Simplicity | `Renderer::new` is `async` which propagates into `CanvasView` via `spawn_local` | Required by the shader-fetch design decision (D1 in tech_spec); `spawn_local` is the standard WASM async bridge |
| Coupling | `draw::create_quad` currently couples geometry upload to program linking | Using `layout(location=0)` in GLSL decouples attribute location from runtime program query |
| Performance | VBO uploaded once; VAO retains bindings — no per-frame allocation | Correct; quad geometry never changes |
| Testability | WebGL init cannot be unit-tested without a browser context | GL init is integration-test territory (`tests/`); pure math tests go in `src/renderer/shader.rs` |

## Implementation Notes

- Use `layout(location = 0) in vec2 a_position` in vert.glsl so `create_quad`
  can hardcode attribute index 0 without querying the program at runtime.
- `glow::Context::from_webgl2_context` is `unsafe`; document the invariant.
- Delete compiled shader objects after linking (saves GPU memory).
- `Drop for Renderer` cleans up all GPU handles.
- `Effect::new(move |_| { ... })` fires once with `None` (pre-mount) and once
  with the canvas element (post-mount); guard with `if let Some`.

## Test Results

_Filled in after implementation._

## Review Notes

- All public `fn`/`struct`/`mod` have `///` or `//!` doc comments.
- No magic numbers: the `12` in `QUAD_VERTS` is explained by the const definition; `6` vertices in `draw_arrays` matches the 2-triangle count.
- No `.unwrap()` calls; all fallible operations return `Result`.
- `map_err` replaced by `inspect_err` where appropriate (clippy caught this — zero warnings).
- `from_webgl2_context` is not `unsafe` in glow 0.13; removed misleading SAFETY comment.

## Test Results

`python make.py build` — ✅ zero warnings  
`python make.py lint` — ✅ zero warnings

Manual test checklist (pending browser verify):
- [ ] Canvas visible at 800×800
- [ ] Solid brass colour rendered
- [ ] No WebGL errors in console
- [ ] Shader fetch 200 OK in network tab
- [ ] `python make.py lint` exits 0 ✅

## Callouts / Gotchas

- `glow::Context::from_webgl2_context` is **not** `unsafe` in glow 0.13 (the
  function signature does not use `unsafe`).  Any SAFETY comment is
  misleading; omit it.
- Clippy `manual_inspect` lint fires when `map_err` is used purely for a side
  effect (logging) without transforming the error.  Use `inspect_err` instead.
- `layout(location = 0)` in the vertex shader eliminates the need to call
  `get_attrib_location` at runtime, which simplifies `create_quad` (no
  program handle needed as a parameter).
