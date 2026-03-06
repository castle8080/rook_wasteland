use std::cell::Cell;
use std::rc::Rc;

use gloo_events::EventListener;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::header::load_file;
use crate::state::{AppState, KaleidoscopeParams};
use crate::renderer;

/// Canvas side length in pixels (matches the `width`/`height` HTML attributes).
const CANVAS_SIZE: f32 = 800.0;

/// Canvas element and WebGL rendering surface.
///
/// Renders an 800 × 800 `<canvas>` that hosts the WebGL 2 context.  On first
/// mount the component:
/// - attaches `dragover` / `drop` / `pointerdown` / `pointermove` event
///   listeners (kept alive indefinitely)
/// - initialises the [`renderer`] singleton synchronously (shaders are
///   embedded in the binary; no network round-trip is required)
///
/// A reactive `Effect` reads `KaleidoscopeParams::snapshot()` (registering
/// all params signals as dependencies) and `AppState.image_loaded`, then
/// calls [`renderer::draw`] on every change so the canvas repaints
/// immediately when any control moves or a new image is loaded.
///
/// Before any image is loaded an instructional overlay is displayed on top of
/// the canvas.
#[component]
pub fn CanvasView() -> impl IntoView {
    let canvas_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();
    let app_state = expect_context::<AppState>();
    let params    = expect_context::<KaleidoscopeParams>();

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

                // Attach all permanent event listeners in one block so that
                // the borrow of `canvas_el` is released before we move below.
                {
                    let canvas_el: &web_sys::HtmlCanvasElement = &canvas;

                    // Drag-and-drop
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

                    // Pointer drag → update centre of symmetry.
                    // `params` is Copy so it is safely captured by each closure.
                    let pparams = params;
                    let pointerdown_listener =
                        EventListener::new(canvas_el, "pointerdown", move |ev| {
                            ev.prevent_default();
                            let ptr: &web_sys::PointerEvent =
                                ev.dyn_ref().expect("PointerEvent");
                            let cx = (ptr.offset_x() as f32 / CANVAS_SIZE).clamp(0.0, 1.0);
                            // Flip Y: HTML y=0 is the top; WebGL v_uv.y=1 is the top.
                            let cy =
                                (1.0 - ptr.offset_y() as f32 / CANVAS_SIZE).clamp(0.0, 1.0);
                            pparams.center.set((cx, cy));
                        });
                    let pointermove_listener =
                        EventListener::new(canvas_el, "pointermove", move |ev| {
                            ev.prevent_default();
                            let ptr: &web_sys::PointerEvent =
                                ev.dyn_ref().expect("PointerEvent");
                            // Only update while the primary button is held.
                            if ptr.buttons() & 1 != 0 {
                                let cx =
                                    (ptr.offset_x() as f32 / CANVAS_SIZE).clamp(0.0, 1.0);
                                let cy = (1.0 - ptr.offset_y() as f32 / CANVAS_SIZE)
                                    .clamp(0.0, 1.0);
                                pparams.center.set((cx, cy));
                            }
                        });

                    // Keep all listeners alive for the entire app lifetime.
                    std::mem::forget(over_listener);
                    std::mem::forget(drop_listener);
                    std::mem::forget(pointerdown_listener);
                    std::mem::forget(pointermove_listener);
                }
                // `canvas_el` borrow released here — canvas can now be moved below.

                // Initialise the renderer synchronously (shaders are embedded).
                match renderer::Renderer::new(&canvas) {
                    Ok(r) => {
                        renderer::set_renderer(r);
                        renderer::draw(&params.snapshot());
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Renderer init failed: {e}").into(),
                        );
                    }
                }
            }
        }
    });

    // --- Reactive draw Effect ------------------------------------------------
    // Reads `image_loaded` and all `KaleidoscopeParams` signals as reactive
    // dependencies so Leptos re-runs this Effect whenever any of them changes.
    Effect::new(move |_| {
        let _image_loaded = app_state.image_loaded.get();
        let snapshot = params.snapshot();
        renderer::draw(&snapshot);
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
