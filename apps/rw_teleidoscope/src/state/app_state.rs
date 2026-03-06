use leptos::prelude::*;

/// Top-level application state (non-render concerns).
/// Provided via context; access with `expect_context::<AppState>()`.
#[derive(Clone, Copy)]
pub struct AppState {
    /// True once an image or camera snapshot has been uploaded to the GPU.
    pub image_loaded: RwSignal<bool>,
    /// True while the camera overlay is open.
    pub camera_open: RwSignal<bool>,
    /// Holds a human-readable camera error message, if any.
    pub camera_error: RwSignal<Option<String>>,
}

impl AppState {
    /// Create a new `AppState` with all flags at their initial values.
    pub fn new() -> Self {
        Self {
            image_loaded: RwSignal::new(false),
            camera_open:  RwSignal::new(false),
            camera_error: RwSignal::new(None),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
