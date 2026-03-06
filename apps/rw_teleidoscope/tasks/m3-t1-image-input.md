# Task M3-T1: Image Input & Texture Display

**Milestone:** M3 — Image Input & Texture Display  
**Status:** 🔄 In Progress

## Restatement

M3 adds the complete image-loading pipeline to the kaleidoscope app.  The user
can select an image via a "Load Image" button (hidden `<input type="file">`) or
by dragging a file onto the canvas.  The selected file is decoded by the browser
via `FileReader.readAsDataURL`, drawn to an offscreen 800 × 800 `<canvas>` for
cover-scaling and crop, and the resulting `ImageData` is uploaded as a WebGL 2
texture that is sampled in the passthrough fragment shader.  `AppState.image_loaded`
is set to `true` to dismiss the placeholder overlay and trigger a redraw via a
reactive `Effect`.  Camera input (M7) and all kaleidoscope transforms (M4–M6) are
explicitly out of scope here.

## Design

### Data flow

```
User picks file (input.change) OR drops file (canvas drop event)
  → validate MIME type ∈ {image/png, image/jpeg, image/webp}
  → FileReader.readAsDataURL(file)
  → ProgressEvent "load" → HtmlImageElement.src = dataURL
  → Event "load" on img
  → utils::resize_to_800(img) → ImageData (800×800 RGBA)
  → renderer::with_renderer_mut(|r| r.upload_image(&image_data))
      → texture::upload_image_data(gl, image_data) → glow::Texture
      → self.source_texture = Some(tex)
  → AppState.image_loaded.set(true)
  → Effect in CanvasView fires (tracks image_loaded) → renderer::draw()
      → draw::draw_frame(gl, program, vao, source_texture, uniform_locs)
```

### Function / type signatures

```rust
// src/utils.rs
/// Compute (dx, dy, draw_w, draw_h) to cover-scale src into a square target.
pub fn cover_rect(img_w: f64, img_h: f64, target: f64) -> (f64, f64, f64, f64);

/// Draw image to an offscreen 800×800 canvas (cover-scaled) and return ImageData.
pub fn resize_to_800(image: &web_sys::HtmlImageElement) -> Result<web_sys::ImageData, String>;

// src/renderer/texture.rs
/// Create a new GL texture from RGBA ImageData; bind to TEXTURE0; return handle.
pub fn upload_image_data(gl: &glow::Context, image_data: &web_sys::ImageData)
    -> Result<glow::Texture, String>;

// src/renderer/uniforms.rs
pub struct UniformLocations { pub u_image: Option<glow::UniformLocation> }
impl UniformLocations {
    pub fn new(gl: &glow::Context, program: glow::Program) -> Self;
    pub fn upload(&self, gl: &glow::Context);  // sets u_image = 0
}

// src/renderer/draw.rs
pub unsafe fn draw_frame(
    gl: &glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    source_texture: Option<glow::Texture>,
    uniform_locs: &UniformLocations,
);

// src/renderer/mod.rs (thread-local RENDERER + free functions)
pub fn set_renderer(renderer: Renderer);
pub fn is_initialized() -> bool;
pub fn with_renderer<F, R>(f: F) -> Option<R> where F: FnOnce(&Renderer) -> R;
pub fn with_renderer_mut<F, R>(f: F) -> Option<R> where F: FnOnce(&mut Renderer) -> R;
pub fn draw();

// On Renderer:
pub fn upload_image(&mut self, image_data: &web_sys::ImageData);

// src/components/header.rs
/// Feed a File through the decode→resize→upload pipeline.
pub fn load_file(file: web_sys::File, app_state: AppState);
```

### Edge cases

- **MIME validation**: only `image/png`, `image/jpeg`, `image/webp` accepted; others
  produce a `console.warn` and leave the canvas unchanged.
- **Re-upload**: calling `upload_image` when a texture already exists must delete
  the old texture first (prevent GPU memory leak).
- **Renderer not yet ready**: `with_renderer_mut` is a no-op if the renderer hasn't
  been initialised; the upload silently fails (renderer init is fast, but this is safe).
- **Zero-dimension image**: `natural_width`/`natural_height` of 0 means `onload` hasn't
  fired yet — the function must only be called inside the `onload` callback.
- **Dropped non-image file**: MIME check in the drop handler prevents pipeline entry.
- **Same file re-selected**: `input.set_value("")` after `load_file` lets the user
  re-select the same file.

### Integration points

| File | Change |
|---|---|
| `Cargo.toml` | Add `HtmlImageElement`, `ImageData`, `DataTransfer` to web-sys features |
| `src/utils.rs` | Implement `cover_rect` + `resize_to_800` |
| `assets/shaders/frag.glsl` | Replace solid colour with `uniform sampler2D u_image` passthrough |
| `src/renderer/texture.rs` | Implement `upload_image_data` |
| `src/renderer/uniforms.rs` | Implement `UniformLocations` + `upload` |
| `src/renderer/draw.rs` | Replace `draw_quad` with `draw_frame` (adds texture/uniform params) |
| `src/renderer/mod.rs` | Add `source_texture`, `uniform_locs` fields; thread-local RENDERER; free fns; `upload_image` |
| `src/components/header.rs` | Implement `Header` + `load_file` pipeline |
| `src/components/canvas_view.rs` | Drag-drop listeners; placeholder overlay; reactive draw Effect |
| `src/app.rs` | Add `<Header/>` to layout |
| `style/main.css` | `canvas-container`, `canvas-placeholder`, header styles |
| `tests/m3_image_input.rs` | Browser integration tests |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `Closure::forget()` leaks memory on every file load | Acceptable for this milestone; each load leaks ~2 small closures (a few hundred bytes), which is negligible for a once-per-session action |
| Simplicity | Thread-local RENDERER adds global state | Necessary because `glow::Context` is `!Send+!Sync` and Leptos context requires `Send+Sync`; scoped tightly to `renderer/mod.rs` |
| Coupling | `canvas_view.rs` calls `header::load_file` for drag-drop | Shared helper function; load_file is `pub` and logically belongs to the loading pipeline regardless of which UI path triggers it |
| Performance | `image_data.data().to_vec()` copies RGBA bytes on every upload | Only happens once per user action (not per frame); 800×800×4 = 2.56 MB is acceptable |
| Testability | `resize_to_800` requires a browser DOM — can't unit-test natively | `cover_rect` is extracted as a pure function with full native `#[test]` coverage; browser path covered by wasm_bindgen_test |

## Implementation Notes

- Use NLL (block scope) to borrow `canvas` for `EventListener::new` and then move it into `spawn_local` — no `Clone` needed.
- `gloo_events::EventListener` clones the `EventTarget` internally so the Rust borrow only needs to last through the `new()` call.
- `std::mem::forget` on both event listeners makes them permanent (no `remove_event_listener` ever fires).
- The reactive draw Effect reads `image_loaded` with `let _x = signal.get()` so Leptos tracks it as a dependency even though the value itself is not used in the draw logic.

## Test Results

_Filled in after Phase 6._

## Review Notes

_Filled in after Phase 7._

## Callouts / Gotchas

_Filled in after Phase 10._
