use leptos::prelude::*;
use crate::error::{AppError, ErrorSeverity};

/// Full-screen blocking overlay shown when the active error has `Fatal` severity.
/// "Start New Game" clears the error signal (full reset logic added in M6).
#[component]
pub fn ErrorOverlay() -> impl IntoView {
    let app_error = use_context::<RwSignal<Option<AppError>>>()
        .expect("app_error context must be provided");

    let is_fatal = move || {
        app_error
            .get()
            .as_ref()
            .map(|e| e.severity() == ErrorSeverity::Fatal)
            .unwrap_or(false)
    };

    view! {
        {move || {
            is_fatal().then(|| {
                let debug_detail = app_error.get().as_ref().map(|e| format!("{e:#?}")).unwrap_or_default();
                view! {
                    <div class="error-overlay">
                        <div class="error-overlay__body">
                            <h2>"⛔ Something went wrong"</h2>
                            <p>
                                "An unexpected error occurred. Your in-progress game \
                                may not be recoverable."
                            </p>
                            <button
                                class="error-overlay__action"
                                on:click=move |_| app_error.set(None)
                            >
                                "Start New Game"
                            </button>
                            {cfg!(debug_assertions).then(|| view! {
                                <details class="error-overlay__details">
                                    <summary>"▶ Details"</summary>
                                    <pre>{debug_detail}</pre>
                                </details>
                            })}
                        </div>
                    </div>
                }
            })
        }}
    }
}
