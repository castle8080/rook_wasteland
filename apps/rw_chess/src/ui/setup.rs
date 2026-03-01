use leptos::prelude::*;
use crate::state::piece::{Color, Difficulty};

/// Game setup / configuration screen.
#[component]
pub fn SetupScreen(on_start: Callback<SetupConfig>) -> impl IntoView {
    let name = RwSignal::new("Player".to_string());
    let color = RwSignal::new(Color::White);
    let difficulty = RwSignal::new(Difficulty::Medium);

    let start = move |_| {
        on_start.run(SetupConfig {
            player_name: name.get(),
            player_color: color.get(),
            difficulty: difficulty.get(),
        });
    };

    view! {
        <div class="setup-screen">
            <div class="setup-card">
                <h2>"♟ Rook Wasteland"</h2>

                // Player name
                <div class="form-group">
                    <label>"Your Name"</label>
                    <input
                        type="text"
                        placeholder="Enter your name"
                        prop:value=move || name.get()
                        on:input:target=move |ev| name.set(ev.target().value())
                    />
                </div>

                // Color selection
                <div class="form-group">
                    <label>"Play as"</label>
                    <div class="button-group">
                        <button
                            class=move || if color.get() == Color::White { "btn selected" } else { "btn" }
                            on:click=move |_| color.set(Color::White)
                        >
                            "♔ White"
                        </button>
                        <button
                            class=move || if color.get() == Color::Black { "btn selected" } else { "btn" }
                            on:click=move |_| color.set(Color::Black)
                        >
                            "♚ Black"
                        </button>
                    </div>
                </div>

                // Difficulty
                <div class="form-group">
                    <label>"Difficulty"</label>
                    <div class="button-group">
                        {[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard]
                            .into_iter()
                            .map(|d| {
                                let label = d.label();
                                view! {
                                    <button
                                        class=move || if difficulty.get() == d { "btn selected" } else { "btn" }
                                        on:click=move |_| difficulty.set(d)
                                    >
                                        {label}
                                    </button>
                                }
                            })
                            .collect_view()
                        }
                    </div>
                </div>

                <button class="btn-primary" on:click=start>
                    "Start Game"
                </button>
            </div>
        </div>
    }
}

#[derive(Clone, Debug)]
pub struct SetupConfig {
    pub player_name: String,
    pub player_color: Color,
    pub difficulty: Difficulty,
}
