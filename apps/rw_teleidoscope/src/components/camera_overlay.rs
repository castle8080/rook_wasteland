use js_sys::Reflect;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::camera;
use crate::renderer;
use crate::state::AppState;

/// Camera live-preview overlay.
///
/// Renders conditionally when `AppState.camera_open` is `true`.  On open it
/// calls [`camera::request_camera`] asynchronously; on permission grant it
/// attaches the `MediaStream` to the `<video>` element via `srcObject`.  The
/// user can then press "Capture" to snap a frame into the kaleidoscope source,
/// or "Cancel" to dismiss without changing the source.
///
/// On error (permission denied or API unavailable) an inline message is shown;
/// no browser `alert()` is used.
#[component]
pub fn CameraOverlay() -> impl IntoView {
    let app_state = expect_context::<AppState>();
    let video_ref: NodeRef<leptos::html::Video> = NodeRef::new();

    // -----------------------------------------------------------------
    // Effect: react to camera_open toggling
    // -----------------------------------------------------------------
    // When camera_open becomes true  → clear any stale error, request camera.
    // When camera_open becomes false → release camera (idempotent).
    Effect::new(move |_| {
        if app_state.camera_open.get() {
            // Clear any stale error from a previous attempt.
            app_state.camera_error.set(None);

            spawn_local(async move {
                match camera::request_camera().await {
                    Ok(stream) => {
                        // If the user closed the overlay while the permission prompt
                        // was showing, do not store the stream — release it immediately.
                        if !app_state.camera_open.get_untracked() {
                            let tracks = stream.get_tracks();
                            for i in 0..tracks.length() {
                                if let Ok(track) = tracks
                                    .get(i)
                                    .dyn_into::<web_sys::MediaStreamTrack>()
                                {
                                    track.stop();
                                }
                            }
                            return;
                        }
                        camera::store_stream(stream.clone());
                        // Attach the stream to the <video> element via srcObject.
                        // We use js_sys::Reflect::set to avoid requiring the
                        // "HtmlMediaElement" web-sys feature.
                        if let Some(video) = video_ref.get_untracked() {
                            let _ = Reflect::set(
                                video.as_ref(),
                                &JsValue::from_str("srcObject"),
                                stream.as_ref(),
                            );
                        }
                    }
                    Err(e) => {
                        app_state.camera_error.set(Some(e));
                    }
                }
            });
        } else {
            camera::release_camera();
        }
    });

    // -----------------------------------------------------------------
    // Button handlers
    // -----------------------------------------------------------------
    let on_capture = move |_| {
        if let Some(video) = video_ref.get_untracked() {
            match camera::capture_frame(&video) {
                Ok(image_data) => {
                    renderer::with_renderer_mut(|r| r.upload_image(&image_data));
                    camera::release_camera();
                    app_state.image_loaded.set(true);
                    app_state.camera_open.set(false);
                }
                Err(e) => {
                    web_sys::console::error_1(&e.clone().into());
                    app_state.camera_error.set(Some(e));
                }
            }
        }
    };

    let on_cancel = move |_| {
        camera::release_camera();
        app_state.camera_open.set(false);
    };

    // -----------------------------------------------------------------
    // View
    // -----------------------------------------------------------------
    view! {
        <Show when=move || app_state.camera_open.get() fallback=|| ()>
            <div class="camera-overlay">
                <div class="camera-preview">
                    <video
                        node_ref=video_ref
                        autoplay=true
                        muted=true
                        playsinline=true
                    />
                    <Show
                        when=move || app_state.camera_error.get().is_some()
                        fallback=|| ()
                    >
                        <p class="camera-error">
                            {move || app_state.camera_error.get().unwrap_or_default()}
                        </p>
                    </Show>
                </div>
                <div class="camera-actions">
                    <button class="header-btn" on:click=on_capture>
                        "📷 CAPTURE"
                    </button>
                    <button class="header-btn" on:click=on_cancel>
                        "✕ CANCEL"
                    </button>
                </div>
            </div>
        </Show>
    }
}
