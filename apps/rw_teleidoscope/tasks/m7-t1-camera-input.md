# Task M7-T1: Camera Input

**Milestone:** M7 — Camera Input  
**Status:** 🔄 In Progress

## Restatement

Implement browser camera access so the user can take a live snapshot and use it
as the kaleidoscope source image.  The feature lives in `src/camera.rs` (pure
camera API) and `src/components/camera_overlay.rs` (UI).  `header.rs` gets a
wired "Use Camera" button; `app.rs` mounts the overlay.  The captured frame is
fed through the same `utils::resize_to_800` → `renderer::upload_image` pipeline
as file input (M3), so the rendering path is unchanged.  This milestone does
**not** implement continuous live streaming — only the single-snapshot pattern.

## Design

### Data flow

1. User clicks "📷 USE CAMERA" in `Header`
2. `AppState.camera_open.set(true)`
3. `CameraOverlay` Effect fires (reacts to `camera_open`) → `spawn_local(camera::request_camera())`
4. Browser permission prompt resolves:
   - `Ok(stream)` → `camera::store_stream(stream.clone())` + `video.srcObject = stream` via `js_sys::Reflect::set`
   - `Err(msg)` → `AppState.camera_error.set(Some(msg))`
5. User clicks "Capture":
   - `camera::capture_frame(&video)` → offscreen 800×800 canvas → `ImageData`
   - `renderer::with_renderer_mut(|r| r.upload_image(&image_data))`
   - `camera::release_camera()` (stops tracks, drops thread-local)
   - `AppState.image_loaded.set(true)`
   - `AppState.camera_open.set(false)` → overlay hides
6. User clicks "Cancel":
   - `camera::release_camera()`
   - `AppState.camera_open.set(false)`
7. Effect re-fires with `camera_open = false` → `camera::release_camera()` (idempotent)

### Function / type signatures

```rust
// camera.rs

/// Request the user's camera via `navigator.mediaDevices.getUserMedia({video: true})`.
/// Returns `Ok(MediaStream)` or a human-readable error string.
pub async fn request_camera() -> Result<web_sys::MediaStream, String>;

/// Store `stream` in the module-level thread-local so it stays alive for the
/// duration of the camera overlay.
pub fn store_stream(stream: web_sys::MediaStream);

/// Stop all tracks on the stored stream and clear the thread-local.
/// Idempotent — safe to call when no stream is stored.
pub fn release_camera();

/// Draw the current video frame onto an offscreen 800×800 canvas (cover-scaled)
/// and return the resulting `ImageData`.
pub fn capture_frame(video: &web_sys::HtmlVideoElement) -> Result<web_sys::ImageData, String>;
```

### Edge cases

- `capture_frame` called when video has zero size (not yet playing) → `cover_rect`
  with 0 dims → produces blank canvas; handled gracefully (no panic).
- `release_camera` called with no stream → thread-local `None` → no-op.
- Permission denied → `JsFuture::from(promise).await` returns `Err(JsValue)` →
  extract `.message` property via `Reflect::get` and surface as `camera_error`.
- Video `srcObject` set before `Show` renders the video element → `video_ref.get_untracked()`
  returns `None` → effect does nothing; subsequent `Effect` re-run is not triggered
  (permission prompt ensures DOM is ready in practice).

### Integration points

| File | Change |
|---|---|
| `src/camera.rs` | Full implementation (was stub) |
| `src/components/camera_overlay.rs` | Full implementation (was stub) |
| `src/components/header.rs` | Wire "Use Camera" button `on:click` handler |
| `src/app.rs` | Import and mount `CameraOverlay` |
| `tests/integration.rs` | Add camera overlay visibility tests |

## Design Critique

| Dimension | Issue | Resolution |
|---|---|---|
| Correctness | `video_ref` may not resolve before `request_camera()` resolves on very fast systems | Permission prompt always takes >0 ms in practice; `get_untracked()` check guards the None case |
| Simplicity | Thread-local for stream mirrors renderer pattern and avoids Send/Sync issues with JsValue | Accepted — same pattern already in use |
| Coupling | `capture_frame` depends on `utils::cover_rect` | Fine — shared utility, no circular dep |
| Performance | Each capture creates a temporary canvas | Acceptable — capture is a one-shot user action |
| Testability | `request_camera` and `capture_frame` require real browser; can't unit test | Integration tests cover signal→DOM wiring; camera API itself covered by manual test checklist |

## Implementation Notes

- Use `js_sys::Reflect::set` for `video.srcObject` to avoid requiring the
  `"HtmlMediaElement"` web-sys feature.
- `Effect::new(|_| { ... })` in `CameraOverlay` reacts to `camera_open`; when true
  it spawns the camera request, when false it releases.
- `NodeRef<leptos::html::Video>` is `Copy`; pass directly into async closures.
- `MediaStream` is a `JsValue` wrapper; `.clone()` is cheap (reference copy).

## Coverage Audit

| Behaviour | Tier | Tested? | Notes |
|---|---|---|---|
| `release_camera` no-op when no stream | 1 | ✅ | Native unit test |
| Overlay hidden when `camera_open = false` | 3 | ✅ | Integration test |
| Overlay shows when `camera_open = true` | 3 | ✅ | Integration test |
| Error message shown when `camera_error` set | 3 | ✅ | Integration test |
| `request_camera()` happy path | — | ❌ waived | Requires real browser permission grant; covered by MT checklist |
| `capture_frame()` happy path | — | ❌ waived | Requires real `HtmlVideoElement` with live frame; covered by MT checklist |
| Capture → `image_loaded = true` | — | ❌ waived | Requires live video; covered by MT checklist |
| Camera light off after capture/cancel | — | ❌ waived | Requires real hardware; manual test MT checklist |

## Test Results

(to be filled in after Phase 6)

## Review Notes

(to be filled in after Phase 7)

## Callouts / Gotchas

- `web_sys::MediaStream` is `!Send`; stored in thread-local to avoid `RwSignal` constraints.
- `video.srcObject` must be set via `js_sys::Reflect::set` to avoid needing the
  `"HtmlMediaElement"` web-sys feature.
- `autoplay`, `playsinline`, and `muted` attributes are required on `<video>` for
  cross-browser autoplay of camera streams.
