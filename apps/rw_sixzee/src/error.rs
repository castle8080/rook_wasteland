use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    /// localStorage access, read, or write failure (includes unavailability).
    #[error("Storage error: {0}")]
    Storage(String),

    /// JSON serialisation or deserialisation failure.
    #[error("JSON error: {0}")]
    Json(String),

    /// Web Worker initialisation or postMessage failure.
    #[error("Ask Grandma worker error: {0}")]
    Worker(String),

    /// Grandma quotes JSON fetch or parse failure.
    #[error("Grandma quotes unavailable: {0}")]
    GrandmaQuotes(String),

    /// A web-sys / DOM API returned an error JsValue.
    #[error("DOM error: {0}")]
    Dom(String),

    /// An internal consistency violation that indicates a programming error.
    #[error("Internal error: {0}")]
    Internal(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Feature degraded but game continues. Show a non-blocking banner.
    Degraded,
    /// Unexpected failure; game state may be unreliable. Show a blocking overlay.
    Fatal,
}

impl AppError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Storage(_) => ErrorSeverity::Degraded,
            AppError::Worker(_) => ErrorSeverity::Degraded,
            AppError::GrandmaQuotes(_) => ErrorSeverity::Degraded,
            AppError::Json(_) => ErrorSeverity::Fatal,
            AppError::Dom(_) => ErrorSeverity::Fatal,
            AppError::Internal(_) => ErrorSeverity::Fatal,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Json(e.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for AppError {
    fn from(v: wasm_bindgen::JsValue) -> Self {
        AppError::Dom(
            v.as_string()
                .unwrap_or_else(|| "(non-string JS error)".to_string()),
        )
    }
}

/// Post an error to the app-level signal. Logs to the browser console in all builds.
#[cfg(target_arch = "wasm32")]
pub fn report_error(err: AppError) {
    use leptos::prelude::{use_context, RwSignal, Set};
    web_sys::console::error_1(&format!("[rw_sixzee] {err:?}").into());
    if let Some(signal) = use_context::<RwSignal<Option<AppError>>>() {
        signal.set(Some(err));
    }
}
