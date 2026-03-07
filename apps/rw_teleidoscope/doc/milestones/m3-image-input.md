# M3 — Image Input & Texture Display

**Status:** 🔄 In Progress  
**Depends on:** [M2 — WebGL Canvas & Basic Renderer](m2-webgl-renderer.md)  
**Unlocks:** [M4 — Mirror Symmetry Core](m4-mirror-symmetry.md)

---

## Goal

A user can load an image from their device (file picker or drag-and-drop) and
see it displayed on the canvas unchanged. The image goes through the full
decode → resize → texture upload → shader sample pipeline. Camera input is not
part of this milestone (see M7).

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Implement `utils.rs` `resize_to_800(image_element)` — draw image to an offscreen 800×800 `<canvas>`, crop/scale to fill, return `ImageData` | ✅ |
| 2 | Implement `renderer/texture.rs` `upload_image_data(gl, image_data)` — create or replace GL texture on unit 0; set `CLAMP_TO_EDGE` / `LINEAR` parameters | ✅ |
| 3 | Update `frag.glsl` — add `uniform sampler2D u_image`; sample it with `v_uv` and output the colour (passthrough) | ✅ |
| 4 | Implement `renderer/uniforms.rs` — `UniformLocations` struct caching all uniform location handles; `upload(gl, params_snapshot)` method (only `u_image` needed for now) | ✅ |
| 5 | Add `Renderer::upload_image(image_data)` calling `texture::upload_image_data` | ✅ |
| 6 | Create `src/components/header.rs` — "Load Image" button that triggers a hidden `<input type="file" accept="image/*">` | ✅ |
| 7 | Wire file input `on:change` handler — `FileReader.readAsDataURL` → create `HtmlImageElement` → `onload` → `resize_to_800` → `renderer.upload_image` → `AppState.image_loaded.set(true)` | ✅ |
| 8 | Implement drag-and-drop on the canvas element — handle `dragover` (prevent default) and `drop` events; extract `File` from `DataTransfer`; run through same pipeline as task 7 | ✅ |
| 9 | Show a placeholder / instructions overlay on the canvas when `AppState.image_loaded` is false | ✅ |
| 10 | Trigger `renderer.draw()` via the `Effect` in `CanvasView` after image upload | ✅ |
| 11 | Verify `python make.py build` and `python make.py lint` still pass | ✅ |

---

## Manual Test Checklist

- [ ] Click "Load Image" → file picker opens → select a PNG → image appears on canvas
- [ ] Select a JPEG → image appears on canvas
- [ ] Select a WebP → image appears on canvas
- [ ] Drag-and-drop a file onto the canvas → image appears
- [ ] Drop a non-image file (e.g. `.txt`) → no crash, canvas unchanged
- [ ] Large image (e.g. 4000×3000 JPEG) → displays at 800×800, no browser freeze
- [ ] Placeholder/instructions visible before any image is loaded
- [ ] No console errors during any of the above

---

## Notes

- Use `FileReader.readAsDataURL` → `HtmlImageElement` src → `onload` rather than
  `readAsArrayBuffer`, as it lets the browser handle all format decoding.
- The offscreen resize canvas does **not** need to be in the DOM.
- Accepted file types: `image/png`, `image/jpeg`, `image/webp` — validate MIME type
  client-side and show an inline error for unsupported types.
- `gloo-events` `EventListener` for drag events on the canvas must be kept alive
  with `std::mem::forget` to span the app lifetime.
