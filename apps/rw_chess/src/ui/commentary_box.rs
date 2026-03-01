use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_timers::future::TimeoutFuture;
use crate::state::game::GameState;

/// Animated speech bubble showing the engine persona's latest commentary.
/// Uses `commentary_gen` so each new line unmounts/remounts the div,
/// re-triggering the CSS entry animation.
#[component]
pub fn CommentaryBox() -> impl IntoView {
    let game = expect_context::<GameState>();

    view! {
        {move || {
            let gen_val = game.commentary_gen.get();
            let text = game.commentary.get()?;
            let name = game.persona.get_untracked().name;
            let avatar = game.persona.get_untracked().avatar;
            let title = game.persona.get_untracked().title;

            // Auto-dismiss: clear after 8s unless a newer gen has already arrived.
            let game_dismiss = game.clone();
            spawn_local(async move {
                TimeoutFuture::new(8_000).await;
                // Only clear if the gen hasn't changed (i.e. no newer commentary)
                if game_dismiss.commentary_gen.get_untracked() == gen_val {
                    game_dismiss.clear_commentary();
                }
            });

            Some(view! {
                <div class="commentary-bubble">
                    <div class="commentary-header">
                        <span class="commentary-avatar">{avatar}</span>
                        <div class="commentary-identity">
                            <strong class="commentary-name">{name}</strong>
                            <span class="commentary-title">{title}</span>
                        </div>
                    </div>
                    <div class="commentary-text">{text}</div>
                    <div class="commentary-tail" />
                </div>
            })
        }}
    }
}
