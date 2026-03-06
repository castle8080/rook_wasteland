//! Camera access, live preview, and frame capture.
//!
//! Provides three public entry points used by [`crate::components::camera_overlay`]:
//! - [`request_camera`] — async; prompts the user for camera permission
//! - [`store_stream`] / [`release_camera`] — keep the `MediaStream` alive while the
//!   overlay is open, then stop all tracks when the overlay closes
//! - [`capture_frame`] — draw a video frame onto an offscreen canvas and return
//!   the raw `ImageData` ready for upload to the GPU

use std::cell::RefCell;

use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::utils;

// ---------------------------------------------------------------------------
// Thread-local stream storage
// ---------------------------------------------------------------------------

thread_local! {
    /// The active `MediaStream`, kept alive for the duration of the camera overlay.
    ///
    /// `web_sys::MediaStream` wraps a `JsValue` and is `!Send + !Sync`, so it
    /// cannot be stored in a `RwSignal`.  A thread-local mirrors the pattern used
    /// by the renderer singleton.
    static CAMERA_STREAM: RefCell<Option<web_sys::MediaStream>> = const { RefCell::new(None) };
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Request the user's camera via `navigator.mediaDevices.getUserMedia({video: true})`.
///
/// Returns `Ok(MediaStream)` on success, or a human-readable `Err(String)` if
/// the user denied permission or the API is unavailable.
pub async fn request_camera() -> Result<web_sys::MediaStream, String> {
    let window = web_sys::window().ok_or("no window")?;
    let navigator = window.navigator();

    let media_devices = navigator
        .media_devices()
        .map_err(|e| format!("navigator.mediaDevices unavailable: {e:?}"))?;

    let constraints = web_sys::MediaStreamConstraints::new();
    constraints.set_video(&JsValue::TRUE);

    let promise = media_devices
        .get_user_media_with_constraints(&constraints)
        .map_err(|e| format!("getUserMedia call failed: {e:?}"))?;

    let js_value = JsFuture::from(promise).await.map_err(|e| {
        // Extract the DOMException `.message` when available.
        let msg = Reflect::get(&e, &JsValue::from_str("message"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| format!("{e:?}"));
        format!("Camera access denied: {msg}")
    })?;

    js_value
        .dyn_into::<web_sys::MediaStream>()
        .map_err(|_| "getUserMedia did not return a MediaStream".to_string())
}

/// Store `stream` in the module-level thread-local so it stays alive while the
/// camera overlay is open.
///
/// If a stream is already stored, its tracks are stopped before the new one is
/// saved — this prevents leaks when rapid open/close cycles overlap.
pub fn store_stream(stream: web_sys::MediaStream) {
    CAMERA_STREAM.with(|s| {
        // Release any previously stored stream before overwriting.
        if let Some(old) = s.borrow().as_ref() {
            let tracks = old.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        *s.borrow_mut() = Some(stream);
    });
}

/// Stop all tracks on the stored `MediaStream` and clear the thread-local.
///
/// Idempotent — safe to call when no stream is stored.  After this call the
/// camera hardware indicator light should turn off.
pub fn release_camera() {
    CAMERA_STREAM.with(|s| {
        if let Some(stream) = s.borrow().as_ref() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                if let Ok(track) = tracks.get(i).dyn_into::<web_sys::MediaStreamTrack>() {
                    track.stop();
                }
            }
        }
        *s.borrow_mut() = None;
    });
}

/// Draw the current video frame onto an offscreen 800 × 800 canvas (cover-scaled,
/// centre-cropped) and return the resulting `ImageData`.
///
/// Returns an error string if the canvas context cannot be obtained or if
/// `drawImage` / `getImageData` fail (e.g. the video has not started playing yet).
pub fn capture_frame(video: &web_sys::HtmlVideoElement) -> Result<web_sys::ImageData, String> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let canvas: web_sys::HtmlCanvasElement = document
        .create_element("canvas")
        .map_err(|e| format!("create canvas: {e:?}"))?
        .dyn_into()
        .map_err(|_| "element is not HtmlCanvasElement".to_string())?;

    const TARGET: u32 = 800;
    canvas.set_width(TARGET);
    canvas.set_height(TARGET);

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .map_err(|e| format!("get_context failed: {e:?}"))?
        .ok_or("no 2d context")?
        .dyn_into()
        .map_err(|_| "not a CanvasRenderingContext2d".to_string())?;

    let vw = f64::from(video.video_width());
    let vh = f64::from(video.video_height());
    let target = f64::from(TARGET);
    let (dx, dy, draw_w, draw_h) = utils::cover_rect(vw, vh, target);

    ctx.draw_image_with_html_video_element_and_dw_and_dh(video, dx, dy, draw_w, draw_h)
        .map_err(|e| format!("drawImage failed: {e:?}"))?;

    ctx.get_image_data(0.0, 0.0, target, target)
        .map_err(|e| format!("getImageData failed: {e:?}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::release_camera;

    #[test]
    fn release_camera_is_noop_when_no_stream() {
        // Should not panic when called with no stored stream.
        release_camera();
        release_camera(); // idempotent
    }
}
