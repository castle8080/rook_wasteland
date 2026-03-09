//! Web Worker entry point for the Ask Grandma computation engine.
//!
//! This module is compiled only when `--features worker` is active.  In that
//! build, the library's normal Leptos start function is suppressed and this
//! `#[wasm_bindgen(start)]` function becomes the sole WASM entry point.
//!
//! The worker:
//!   1. Posts `"ready"` once initialised so callers know it is safe to send requests.
//!   2. Listens for `message` events on the `DedicatedWorkerGlobalScope`.
//!   3. Deserialises each `GrandmaRequest`, runs `compute_grandma_actions`, and
//!      posts back the serialised `GrandmaResponse`.

#[cfg(all(target_arch = "wasm32", feature = "worker"))]
mod inner {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    use crate::worker::advisor::compute_grandma_actions;
    use crate::worker::messages::{GrandmaRequest, GrandmaResponse};

    /// Worker entry point — called automatically when the WASM module is
    /// instantiated in the `DedicatedWorkerGlobalScope`.
    #[wasm_bindgen(start)]
    pub fn worker_start() {
        console_error_panic_hook::set_once();

        let global = js_sys::global()
            .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
            .expect("grandma_worker must run inside a DedicatedWorkerGlobalScope");

        // Register the message handler.
        let global_for_handler = global.clone();
        let handler =
            Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
                let data = e.data();
                let response: GrandmaResponse = match serde_wasm_bindgen::from_value::<
                    GrandmaRequest,
                >(data)
                {
                    Ok(req) => compute_grandma_actions(&req),
                    Err(err) => {
                        // Post a serialised error object so the main thread can surface it.
                        let err_val = JsValue::from_str(&format!("deserialise error: {err}"));
                        let _ = global_for_handler.post_message(&err_val);
                        return;
                    }
                };

                match serde_wasm_bindgen::to_value(&response) {
                    Ok(val) => {
                        let _ = global_for_handler.post_message(&val);
                    }
                    Err(err) => {
                        let err_val =
                            JsValue::from_str(&format!("serialise error: {err}"));
                        let _ = global_for_handler.post_message(&err_val);
                    }
                }
            });

        global.set_onmessage(Some(handler.as_ref().unchecked_ref()));
        handler.forget(); // Worker lives for the page lifetime.

        // Signal that the worker is initialised and ready to receive requests.
        let _ = global.post_message(&JsValue::from_str("ready"));
    }
}
