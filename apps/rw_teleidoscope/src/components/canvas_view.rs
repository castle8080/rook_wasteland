use std::cell::Cell;
use std::rc::Rc;

use gloo_events::EventListener;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::components::header::load_file;
use crate::state::AppState;
use crate::renderer;

/// Canvas element and WebGL rendering surface.
///
/// Renders an 800 × 800 `<canvas>` that hosts the WebGL 2 context.  On first
/// mount the component:
/// - attaches `dragover` / `drop` event listeners (kept alive indefinitely)
/// - initialises the [`renderer`] singleton asynchronously
///
/// A separate reactive `Effect` watches `AppState.image_loaded` and calls
/// [`renderer::draw`] whenever it changes, so the canvas repaints as soon as a
/// new image is uploaded.
///
/// Before any image is loaded an instructional overlay is displayed on top of
/// the canvas.
#[component]
pub fn CanvasView() -> impl IntoView {
    let canvas_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();
    let app_state = expect_context::<AppState>();

    // Guard: ensure the one-time initialisation block runs only once even if
    // the Effect re-fires (Leptos fires Effects twice in development mode).
    let initialized = Rc::new(Cell::new(false));

    // --- Initialisation Effect -----------------------------------------------
    // Fires once the canvas NodeRef resolves (i.e. after the first mount).
    Effect::new({
        let initialized = Rc::clone(&initialized);
        move |_| {
            if let Some(canvas) = canvas_ref.get() {
                if initialized.get() {
                    return;
                }
                initialized.set(true);

                // Attach drag-and-drop listeners permanently.
                // The borrow of `canvas` (via &*canvas) ends after EventListener::new()
                // returns because EventListener clones the EventTarget internally.
                {
                    let canvas_el: &web_sys::HtmlCanvasElement = &canvas;
                    let over_listener = EventListener::new(canvas_el, "dragover", |ev| {
                        ev.prevent_default();
                    });
                    let drop_listener =
                        EventListener::new(canvas_el, "drop", move |ev| {
                            ev.prevent_default();
                            let drop_ev: &web_sys::DragEvent =
                                ev.dyn_ref().expect("DragEvent");
                            if let Some(dt) = drop_ev.data_transfer() {
                                if let Some(files) = dt.files() {
                                    if let Some(file) = files.get(0) {
                                        load_file(file, app_state);
                                    }
                                }
                            }
                        });
                    // Keep alive for the entire app lifetime.
                    std::mem::forget(over_listener);
                    std::mem::forget(drop_listener);
                }
                // `canvas_el` borrow released here — canvas can now be moved below.

                // Initialise the renderer asynchronously (shader fetch).
                spawn_local(async move {
                    match renderer::Renderer::new(&canvas).await {
                        Ok(r) => {
                            renderer::set_renderer(r);
                            renderer::draw();
                        }
                        Err(e) => {
                            web_sys::console::error_1(
                                &format!("Renderer init failed: {e}").into(),
                            );
                        }
                    }
                });
            }
        }
    });

    // --- Reactive draw Effect ------------------------------------------------
    // Reads `image_loaded` as a reactive dependency so Leptos re-runs this
    // Effect whenever an image is uploaded (or unloaded in future milestones).
    Effect::new(move |_| {
        let _image_loaded = app_state.image_loaded.get();
        renderer::draw();
    });

    view! {
        <div class="canvas-container">
            <canvas
                node_ref=canvas_ref
                id="kaleidoscope-canvas"
                width="800"
                height="800"
            />
            // Overlay shown only before any image has been loaded.
            <Show when=move || !app_state.image_loaded.get()>
                <div class="canvas-placeholder">
                    <div class="placeholder-inner">
                        <span class="placeholder-icon">"⚙"</span>
                        <p>"Drop an image here, or use"</p>
                        <p>"Load Image / Use Camera above."</p>
                        <p class="placeholder-formats">"Supported: PNG  JPEG  WebP"</p>
                    </div>
                </div>
            </Show>
        </div>
    }
}


