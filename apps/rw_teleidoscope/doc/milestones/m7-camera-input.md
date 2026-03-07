# M7 — Camera Input

**Status:** ✅ Complete  
**Depends on:** [M4 — Mirror Symmetry Core](m4-mirror-symmetry.md)  
**Unlocks:** [M10 — Steampunk Polish](m10-steampunk-polish.md) (after M9)

---

## Goal

A user can tap "Use Camera", see a live video preview in an overlay, press
"Capture" to freeze a frame as the kaleidoscope source image, or "Cancel" to
dismiss. If camera permission is denied the app shows a clear inline message
and does not crash. The captured frame feeds through the exact same
resize → texture upload pipeline as M3 file input.

---

## Tasks

| # | Task | Status |
|---|---|---|
| 1 | Implement `camera::request_camera()` — call `navigator.media_devices().get_user_media(constraints)`, bridge the JS Promise with `JsFuture`, return `Ok(MediaStream)` or `Err(human-readable String)` | ✅ |
| 2 | Implement `camera::release_camera(stream)` — iterate `stream.get_tracks()`, call `track.stop()` on each | ✅ |
| 3 | Implement `camera::capture_frame(video)` — create offscreen 800×800 `<canvas>`, draw video frame with `draw_image_with_html_video_element_and_dw_and_dh`, call `get_image_data`, return `ImageData` | ✅ |
| 4 | Create `src/components/camera_overlay.rs` — renders conditionally on `AppState.camera_open`; contains `<video autoplay>` element and Capture / Cancel buttons | ✅ |
| 5 | On `CameraOverlay` mount: call `spawn_local(request_camera())` → on `Ok`, set `video.set_src_object(Some(&stream))`; on `Err`, write to `AppState.camera_error` | ✅ |
| 6 | Implement error state display in `CameraOverlay` — show inline error message from `AppState.camera_error`; no browser `alert()` | ✅ |
| 7 | Implement "Capture" button handler — call `capture_frame(video)`, pass `ImageData` to `renderer.upload_image()`, call `release_camera(stream)`, set `AppState.camera_open.set(false)`, set `AppState.image_loaded.set(true)` | ✅ |
| 8 | Implement "Cancel" button handler — call `release_camera(stream)`, set `AppState.camera_open.set(false)` | ✅ |
| 9 | Add "Use Camera" button to `header.rs`; sets `AppState.camera_open.set(true)` | ✅ |
| 10 | Ensure `CameraOverlay` is mounted in `App` layout, overlaid on the canvas | ✅ |
| 11 | Verify `python make.py build` and `python make.py lint` pass | ✅ |

---

## Manual Test Checklist

- [ ] Click "Use Camera" → browser permission prompt appears
- [ ] Grant permission → live video preview appears in overlay
- [ ] Click "Capture" → overlay closes; captured frame appears as kaleidoscope source
- [ ] Captured image looks reasonable (not black/corrupt/wrong size)
- [ ] Click "Cancel" → overlay closes; previous source image (if any) unchanged
- [ ] Deny camera permission → overlay shows inline error message (no browser alert, no crash)
- [ ] After capture, camera light on device turns off (tracks stopped)
- [ ] No console errors during any of the above

---

## Notes

- `getUserMedia` is only available in secure contexts (HTTPS or localhost).
  During development `trunk serve` on localhost is fine.
- `web-sys` `MediaDevices::get_user_media_with_constraints` returns a `Promise`
  — wrap with `wasm_bindgen_futures::JsFuture::from` to `await` in Rust.
- The `<video>` element must have `autoplay` and `playsinline` attributes set,
  or the stream may not render on some browsers/devices.
- Keep the `MediaStream` alive in component state for the duration of the overlay.
  Dropping it too early stops the camera feed.
- This milestone does **not** implement live continuous streaming — only the
  snapshot capture pattern (FR-4 through FR-9 in PRD Section 5.1).
