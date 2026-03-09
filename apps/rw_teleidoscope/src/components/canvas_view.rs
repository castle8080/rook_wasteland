use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gloo_events::{EventListener, EventListenerOptions};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::header::load_file;
use crate::state::{AppState, KaleidoscopeParams};
use crate::utils::{apply_pinch_zoom, pinch_distance};
use crate::renderer;

/// Minimum and maximum zoom values enforced by the pinch gesture.
const PINCH_ZOOM_MIN: f32 = 0.25;
const PINCH_ZOOM_MAX: f32 = 5.0;

/// Canvas element and WebGL rendering surface.
///
/// Renders an 800 × 800 `<canvas>` that hosts the WebGL 2 context.  The HTML
/// `width`/`height` attributes fix the drawing buffer at 800 × 800; CSS scales
/// the canvas to fill the available space via `max-width: 100%; height: auto`.
/// Pointer coordinate normalisation uses `client_width()` / `client_height()` so
/// coordinates are correct at any CSS display size (no hard-coded 800 constant).
///
/// On first mount the component:
/// - attaches `dragover` / `drop` / `pointerdown` / `pointermove` /
///   `pointerup` / `pointercancel` event listeners (kept alive indefinitely)
/// - initialises the [`renderer`] singleton synchronously (shaders are
///   embedded in the binary; no network round-trip is required)
///
/// Single-finger drag (or mouse drag) updates `KaleidoscopeParams::center`.
/// Two-finger pinch updates `KaleidoscopeParams::zoom` in the range 0.25 – 5.0.
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

                    // Drag-and-drop — must be non-passive so prevent_default() works
                    // (prevents the browser from overriding the drop with its default
                    // file-open behaviour).
                    let opts = EventListenerOptions::enable_prevent_default();
                    let over_listener = EventListener::new_with_options(canvas_el, "dragover", opts, |ev| {
                        ev.prevent_default();
                    });
                    let drop_listener =
                        EventListener::new_with_options(canvas_el, "drop", opts, move |ev| {
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
                    // Two-finger pinch → update zoom.
                    //
                    // All pointer listeners are non-passive so prevent_default()
                    // can suppress text selection and touchscreen scrolling.
                    // `params` is Copy so it is safely captured by each closure.
                    //
                    // Pointer tracking: active_pointers maps pointer_id to the
                    // most recent (offset_x, offset_y) for that contact point.
                    // last_pinch_dist records the reference distance from the
                    // previous frame so the zoom delta can be computed.
                    let pparams = params;
                    let active_pointers: Rc<RefCell<HashMap<i32, (f32, f32)>>> =
                        Rc::new(RefCell::new(HashMap::new()));
                    let last_pinch_dist: Rc<Cell<f32>> = Rc::new(Cell::new(0.0));

                    // Clone the canvas handle so it can be moved into closures
                    // (web_sys types are JS reference-counted; clone is cheap).
                    let canvas_pd = (*canvas_el).clone();
                    let canvas_pm = (*canvas_el).clone();

                    let ap_down = Rc::clone(&active_pointers);
                    let lpd_down = Rc::clone(&last_pinch_dist);
                    let pointerdown_listener =
                        EventListener::new_with_options(canvas_el, "pointerdown", opts, move |ev| {
                            ev.prevent_default();
                            let ptr: &web_sys::PointerEvent =
                                ev.dyn_ref().expect("PointerEvent");
                            let pid = ptr.pointer_id();

                            // Capture ensures drag events keep firing even if
                            // the pointer moves outside the canvas boundary.
                            let _ = canvas_pd.set_pointer_capture(pid);

                            let ox = ptr.offset_x() as f32;
                            let oy = ptr.offset_y() as f32;
                            ap_down.borrow_mut().insert(pid, (ox, oy));

                            let count = ap_down.borrow().len();
                            if count == 1 {
                                // Single contact: update centre of symmetry.
                                let w = canvas_pd.client_width().max(1) as f32;
                                let h = canvas_pd.client_height().max(1) as f32;
                                let cx = (ox / w).clamp(0.0, 1.0);
                                // Flip Y: HTML y=0 is top; WebGL v_uv.y=1 is top.
                                let cy = (1.0 - oy / h).clamp(0.0, 1.0);
                                pparams.center.set((cx, cy));
                            } else if count >= 2 {
                                // Two contacts: record initial pinch distance.
                                let positions: Vec<(f32, f32)> =
                                    ap_down.borrow().values().copied().collect();
                                let d = pinch_distance(
                                    positions[0].0, positions[0].1,
                                    positions[1].0, positions[1].1,
                                );
                                lpd_down.set(d);
                            }
                        });

                    let ap_move = Rc::clone(&active_pointers);
                    let lpd_move = Rc::clone(&last_pinch_dist);
                    let pointermove_listener =
                        EventListener::new_with_options(canvas_el, "pointermove", opts, move |ev| {
                            let ptr: &web_sys::PointerEvent =
                                ev.dyn_ref().expect("PointerEvent");
                            let pid = ptr.pointer_id();

                            // Only process pointers that were registered on pointerdown.
                            if !ap_move.borrow().contains_key(&pid) {
                                return;
                            }

                            let ox = ptr.offset_x() as f32;
                            let oy = ptr.offset_y() as f32;
                            ap_move.borrow_mut().insert(pid, (ox, oy));

                            let count = ap_move.borrow().len();
                            if count == 1 {
                                // Single-contact drag: update centre only while
                                // the primary button (or touch) is active.
                                if ptr.buttons() & 1 != 0 {
                                    ev.prevent_default();
                                    let w = canvas_pm.client_width().max(1) as f32;
                                    let h = canvas_pm.client_height().max(1) as f32;
                                    let cx = (ox / w).clamp(0.0, 1.0);
                                    let cy = (1.0 - oy / h).clamp(0.0, 1.0);
                                    pparams.center.set((cx, cy));
                                }
                            } else if count >= 2 {
                                // Two-contact pinch: update zoom.
                                ev.prevent_default();
                                let positions: Vec<(f32, f32)> =
                                    ap_move.borrow().values().copied().collect();
                                let new_dist = pinch_distance(
                                    positions[0].0, positions[0].1,
                                    positions[1].0, positions[1].1,
                                );
                                let old_dist = lpd_move.get();
                                let new_zoom = apply_pinch_zoom(
                                    old_dist, new_dist,
                                    pparams.zoom.get_untracked(),
                                    PINCH_ZOOM_MIN, PINCH_ZOOM_MAX,
                                );
                                pparams.zoom.set(new_zoom);
                                lpd_move.set(new_dist);
                            }
                        });

                    let ap_up = Rc::clone(&active_pointers);
                    let lpd_up = Rc::clone(&last_pinch_dist);
                    let pointerup_listener =
                        EventListener::new_with_options(canvas_el, "pointerup", opts, move |ev| {
                            let ptr: &web_sys::PointerEvent =
                                ev.dyn_ref().expect("PointerEvent");
                            ap_up.borrow_mut().remove(&ptr.pointer_id());
                            if ap_up.borrow().len() < 2 {
                                lpd_up.set(0.0);
                            }
                        });

                    let ap_cancel = Rc::clone(&active_pointers);
                    let lpd_cancel = Rc::clone(&last_pinch_dist);
                    let pointercancel_listener =
                        EventListener::new_with_options(canvas_el, "pointercancel", opts, move |ev| {
                            let ptr: &web_sys::PointerEvent =
                                ev.dyn_ref().expect("PointerEvent");
                            ap_cancel.borrow_mut().remove(&ptr.pointer_id());
                            // Always reset on cancel (e.g. notification shade, incoming call).
                            lpd_cancel.set(0.0);
                        });

                    // Keep all listeners alive for the entire app lifetime.
                    std::mem::forget(over_listener);
                    std::mem::forget(drop_listener);
                    std::mem::forget(pointerdown_listener);
                    std::mem::forget(pointermove_listener);
                    std::mem::forget(pointerup_listener);
                    std::mem::forget(pointercancel_listener);
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
