use leptos::prelude::*;

/// Resume-vs-new-game prompt overlay (placeholder — logic implemented in M6).
#[component]
pub fn ResumePrompt() -> impl IntoView {
    let show_resume =
        use_context::<RwSignal<bool>>().expect("show_resume context must be provided");

    view! {
        <div class="overlay">
            <div class="overlay__box resume-prompt">
                <h2>"Resume Game?"</h2>
                <p>"A game is in progress."</p>
                <div class="resume-prompt__actions">
                    <button on:click=move |_| show_resume.set(false)>
                        "Resume"
                    </button>
                    <button on:click=move |_| show_resume.set(false)>
                        "New Game"
                    </button>
                </div>
            </div>
        </div>
    }
}
