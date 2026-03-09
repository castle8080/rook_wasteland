use leptos::prelude::*;
use crate::error::{AppError, ErrorSeverity};

/// Dismissible banner shown when the active error has `Degraded` severity.
#[component]
pub fn ErrorBanner() -> impl IntoView {
    let app_error = use_context::<RwSignal<Option<AppError>>>()
        .expect("app_error context must be provided");

    let is_degraded = move || {
        app_error
            .get()
            .as_ref()
            .map(|e| e.severity() == ErrorSeverity::Degraded)
            .unwrap_or(false)
    };

    view! {
        {move || {
            is_degraded().then(|| {
                let message = app_error.get().as_ref().map(|e| e.to_string()).unwrap_or_default();
                let debug_detail = app_error.get().as_ref().map(|e| format!("{e:?}")).unwrap_or_default();
                view! {
                    <div class="error-banner">
                        <span class="error-banner__message">"⚠ " {message}</span>
                        <button
                            class="error-banner__dismiss"
                            on:click=move |_| app_error.set(None)
                        >
                            "✕"
                        </button>
                        {cfg!(debug_assertions).then(|| view! {
                            <details class="error-banner__details">
                                <summary>"Details"</summary>
                                <pre>{debug_detail}</pre>
                            </details>
                        })}
                    </div>
                }
            })
        }}
    }
}
