# M2 — WebGL Canvas & Basic Renderer

**Status:** ⬜ Pending  
**Depends on:** [M1 — Project Scaffold](m1-scaffold.md)  
**Unlocks:** [M3 — Image Input & Texture Display](m3-image-input.md)

---

## Goal

Get the WebGL 2 pipeline running end-to-end: obtain a `glow::Context` from the
canvas, compile the vertex and fragment shaders loaded from `.glsl` files,
draw a full-screen quad, and output a solid colour to the canvas.
No image, no uniforms — just proof that WebGL is initialised correctly and
the shader compilation/fetch pipeline works.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Implement `renderer/context.rs` — obtain `WebGl2RenderingContext` from a canvas `NodeRef` via `web-sys`; wrap with `glow::Context::from_webgl2_context` | ⬜ |
| 2 | Implement `renderer/shader.rs` — fetch `vert.glsl` and `frag.glsl` from Trunk-served `assets/shaders/` at WASM startup using `wasm-bindgen-futures` + `web-sys` `fetch` | ⬜ |
| 3 | Write `assets/shaders/vert.glsl` — full-screen quad: two triangles covering clip-space −1..1, pass through `v_uv` | ⬜ |
| 4 | Write `assets/shaders/frag.glsl` — output a solid steampunk-brass colour (`vec4(0.545, 0.412, 0.078, 1.0)`) so success is visually obvious | ⬜ |
| 5 | Implement `renderer/draw.rs` — allocate VAO and VBO for the quad (uploaded once); implement `draw()` binding the program and issuing `gl.draw_arrays` | ⬜ |
| 6 | Implement `Renderer::new(canvas)` in `renderer/mod.rs` — calls context, fetches shaders, compiles program, allocates quad geometry | ⬜ |
| 7 | Create `src/components/canvas_view.rs` — renders `<canvas>` element; on mount (via `create_effect` / `on_mount`) calls `Renderer::new()` and stores result in `Rc<RefCell<Option<Renderer>>>` | ⬜ |
| 8 | Wire `CanvasView` into `App` so it renders in the page | ⬜ |
| 9 | Add a stub `Effect` in `CanvasView` that calls `renderer.draw()` once (no params yet) | ⬜ |
| 10 | Verify `python make.py build` and `python make.py lint` still pass | ⬜ |

---

## Manual Test Checklist

- [ ] Page loads; canvas element is visible in the browser
- [ ] Canvas displays the solid brass colour (no black square / no white)
- [ ] No WebGL errors in the browser console (`gl.getError()` returns 0)
- [ ] Shader fetch URL is correct (`/rw_teleidoscope/assets/shaders/vert.glsl` loads 200 OK in network tab)
- [ ] `python make.py lint` exits 0

---

## Notes

- `Renderer` is `!Send` due to `glow::Context`. **Never** place it inside a
  Leptos `RwSignal`. Store as `Rc<RefCell<Option<Renderer>>>` in the component.
- Shader fetch must happen before `Renderer::new()` returns — use an `async`
  function bridged with `spawn_local` (`wasm-bindgen-futures`).
- If shader compilation fails, surface the GLSL info log via `web_sys::console::error_1`
  so it appears in the browser dev tools console.
- The quad geometry never changes — upload VBO once in `new()`, reuse every frame.
