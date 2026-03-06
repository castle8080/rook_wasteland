use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::renderer::Renderer;

/// Canvas element and WebGL rendering surface.
///
/// Renders a `<canvas>` element at 800 × 800 px.  On mount the component
/// obtains a WebGL 2 context, fetches and compiles the GLSL shaders, and
/// issues a first draw call.  The renderer is stored as
/// `Rc<RefCell<Option<Renderer>>>` because `glow::Context` is `!Send` and
/// must not live inside a Leptos signal.
#[component]
pub fn CanvasView() -> impl IntoView {
    let canvas_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();
    let renderer: Rc<RefCell<Option<Renderer>>> = Rc::new(RefCell::new(None));
    let renderer_clone = Rc::clone(&renderer);

    // Fires once before mount (canvas_ref is None → no-op) and once after
    // mount (canvas_ref is Some → initialise renderer asynchronously).
    Effect::new(move |_| {
        if let Some(canvas) = canvas_ref.get() {
            // Guard: only initialise once even if the effect re-fires.
            if renderer_clone.borrow().is_some() {
                return;
            }
            let renderer = Rc::clone(&renderer_clone);
            spawn_local(async move {
                match Renderer::new(&canvas).await {
                    Ok(r) => {
                        r.draw();
                        *renderer.borrow_mut() = Some(r);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Renderer init failed: {e}").into(),
                        );
                    }
                }
            });
        }
    });

    view! {
        <canvas
            node_ref=canvas_ref
            id="kaleidoscope-canvas"
            width="800"
            height="800"
        />
    }
}

