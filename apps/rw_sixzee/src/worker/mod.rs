//! Web Worker bridge for Ask Grandma computation.
//!
//! The worker is built as a separate WASM binary (same crate, `--features worker`).
//! The main app spawns it via [`spawn_grandma_worker`] and communicates via
//! serialised [`GrandmaRequest`] / [`GrandmaResponse`] messages.

pub mod advisor;
pub mod grandma_worker;
pub mod messages;

use crate::error::{AppError, AppResult};
use crate::worker::messages::{GrandmaAction, GrandmaRequest, GrandmaResponse};

// ─── Panel state ──────────────────────────────────────────────────────────────

/// State of the Ask Grandma advice panel, held in Leptos context.
///
/// Transitions: `Closed` → `Loading` (on button click) → `Ready` (response) or `Error`.
#[derive(Clone, Debug)]
pub enum GrandmaPanelState {
    /// Panel is closed; no in-flight request.
    Closed,
    /// Request posted; awaiting response.
    Loading,
    /// Response received; actions are ready to display.
    Ready(Vec<GrandmaAction>),
    /// Worker returned an error.
    Error(String),
}

// ─── Worker bridge (wasm32 only) ──────────────────────────────────────────────

/// Spawn the grandma worker, wire its `onmessage` response handler, and store
/// the `Worker` handle in `worker_sig`.
///
/// The response handler updates `panel_state`:
/// - A serialised `GrandmaResponse` → `GrandmaPanelState::Ready(actions)`
/// - A string `"ready"` → ignored (just an initialisation ping)
/// - Any other message → `GrandmaPanelState::Error(msg)`
///
/// Returns `AppError::Worker` if `Worker::new` fails (no Worker API, or CSP blocks it).
#[cfg(target_arch = "wasm32")]
pub fn spawn_grandma_worker(
    worker_sig: leptos::prelude::RwSignal<Option<web_sys::Worker>>,
    panel_state: leptos::prelude::RwSignal<GrandmaPanelState>,
) -> AppResult<()> {
    use leptos::prelude::Set;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    let worker = web_sys::Worker::new("./grandma_worker.js").map_err(|e| {
        AppError::Worker(
            e.as_string()
                .unwrap_or_else(|| "Worker::new failed".to_string()),
        )
    })?;

    let cb =
        Closure::<dyn Fn(web_sys::MessageEvent)>::new(move |e: web_sys::MessageEvent| {
            let data = e.data();

            // Ignore the "ready" ping from worker_start().
            if data.as_string().map(|s| s == "ready").unwrap_or(false) {
                return;
            }

            // Deserialise as GrandmaResponse; surface errors in the panel.
            match serde_wasm_bindgen::from_value::<GrandmaResponse>(data) {
                Ok(resp) => panel_state.set(GrandmaPanelState::Ready(resp.actions)),
                Err(err) => {
                    panel_state.set(GrandmaPanelState::Error(format!(
                        "Could not reach Grandma — {err}"
                    )));
                }
            }
        });

    worker.set_onmessage(Some(cb.as_ref().unchecked_ref()));
    cb.forget(); // Callback lives as long as the worker.

    worker_sig.set(Some(worker));
    Ok(())
}

/// Serialise `req` and post it to the worker's message port.
///
/// Returns `AppError::Worker` if serialisation or `postMessage` fails.
#[cfg(target_arch = "wasm32")]
pub fn post_grandma_request(worker: &web_sys::Worker, req: &GrandmaRequest) -> AppResult<()> {
    let val = serde_wasm_bindgen::to_value(req)
        .map_err(|e| AppError::Worker(format!("request serialise error: {e}")))?;
    worker
        .post_message(&val)
        .map_err(|e| AppError::Worker(e.as_string().unwrap_or_else(|| "postMessage failed".to_string())))
}
