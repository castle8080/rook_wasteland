//! Settings screen — theme picker with live preview and persistence.
//!
//! Reads `ActiveTheme` from context.  Clicking a theme card immediately
//! updates the reactive signal, which triggers the Effect in `App` to apply
//! the new `data-theme` on `<body>` and persist to localStorage.

use leptos::prelude::*;

use crate::dice_svg::DiceFace;
use crate::state::{ActiveTheme, Theme};

/// Settings screen component.
///
/// Shows a 2-column theme picker grid.  Each card previews the die face for
/// value 6 in that theme's style by applying `data-theme` to the card wrapper
/// (so CSS custom properties resolve correctly even for non-active themes).
#[component]
pub fn SettingsView() -> impl IntoView {
    let active_theme =
        use_context::<ActiveTheme>().expect("ActiveTheme context must be provided");

    view! {
        <div class="settings">
            <h2 class="settings__section-title">"THEME"</h2>
            <p class="settings__hint">"Theme applies instantly — no reload needed."</p>
            <div class="settings__theme-grid">
                {Theme::all()
                    .iter()
                    .map(|&t| theme_card(t, active_theme))
                    .collect_view()}
            </div>
        </div>
    }
}

/// Render a single theme card.
fn theme_card(theme: Theme, active_theme: ActiveTheme) -> impl IntoView {
    let attr_val = theme.as_data_attr_value();
    let label = theme.label();

    let is_active = move || active_theme.0.get() == theme;

    let card_class = move || {
        if is_active() {
            "settings__theme-card settings__theme-card--active"
        } else {
            "settings__theme-card"
        }
    };

    let on_click = move |_| {
        active_theme.0.set(theme);
    };

    view! {
        <button
            class=card_class
            data-theme=attr_val
            on:click=on_click
            aria-label=move || format!("Select {} theme{}", label, if is_active() { " (active)" } else { "" })
        >
            <div class="settings__theme-card__die">
                <DiceFace theme=theme value=6 />
            </div>
            <div class="settings__theme-card__swatch" />
            <div class="settings__theme-card__label">{label}</div>
            {move || is_active().then(|| view! { <span class="settings__theme-card__check">"✓"</span> })}
        </button>
    }
}

