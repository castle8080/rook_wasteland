//! Game menu icon and dropdown panel.
//!
//! Renders a ⋮ button in the game header. Tapping it opens a small options
//! panel. The only initial item is "Quit Game". Tapping a backdrop div or the
//! item itself closes the panel.

use leptos::prelude::*;

/// ⋮ menu icon with a dropdown options panel.
///
/// `on_quit_requested` is called when the player taps "Quit Game".
#[component]
pub fn GameMenu(
    /// Called when the player selects "Quit Game" from the panel.
    on_quit_requested: Callback<()>,
) -> impl IntoView {
    let menu_open: RwSignal<bool> = RwSignal::new(false);

    let open = move |_| menu_open.set(true);
    let close = move |_| menu_open.set(false);

    let on_quit = move |_| {
        menu_open.set(false);
        on_quit_requested.run(());
    };

    view! {
        <div class="game-menu">
            <button
                class="game-menu__btn"
                aria-label="Game menu"
                on:click=open
            >
                // U+22EE VERTICAL ELLIPSIS — distinct from the ⚙️ Settings tab icon.
                "⋮"
            </button>
            {move || {
                if menu_open.get() {
                    view! {
                        // Backdrop: clicking outside the panel closes the menu.
                        <div class="game-menu__backdrop" on:click=close />
                        <div class="game-menu__panel">
                            <button
                                class="game-menu__item game-menu__item--danger"
                                on:click=on_quit
                            >
                                "Quit Game"
                            </button>
                        </div>
                    }
                    .into_any()
                } else {
                    view! { <span /> }.into_any()
                }
            }}
        </div>
    }
}
