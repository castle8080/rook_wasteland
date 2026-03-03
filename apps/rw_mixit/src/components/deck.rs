use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::AudioContext;

use crate::audio::{ensure_audio_context, deck_audio::AudioDeck};
use crate::audio::loader::load_audio_file;
use crate::components::mixer::Mixer;
use crate::state::DeckState;

/// Three-column layout: `[Deck A] [Mixer] [Deck B]`.
#[component]
pub fn DeckView() -> impl IntoView {
    let audio_ctx_holder: Rc<RefCell<Option<AudioContext>>> = Rc::new(RefCell::new(None));

    let state_a = DeckState::new();
    let state_b = DeckState::new();

    view! {
        <div class="deck-row">
            <Deck side="A" state=state_a audio_ctx_holder=audio_ctx_holder.clone()/>
            <Mixer/>
            <Deck side="B" state=state_b audio_ctx_holder=audio_ctx_holder/>
        </div>
    }
}

/// A single DJ deck with Load Track button and track info display.
#[component]
pub fn Deck(
    side: &'static str,
    state: DeckState,
    audio_ctx_holder: Rc<RefCell<Option<AudioContext>>>,
) -> impl IntoView {
    // AudioDeck is created lazily on first Load click (requires user gesture for AudioContext)
    let audio_deck: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>> =
        Rc::new(RefCell::new(None));

    let file_input: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_load_click = {
        move |_: web_sys::MouseEvent| {
            if let Some(input) = file_input.get() {
                input.click();
            }
        }
    };

    let on_file_change = {
        let audio_ctx_holder = audio_ctx_holder.clone();
        let audio_deck = audio_deck.clone();
        let state = state.clone();
        move |ev: web_sys::Event| {
            let input: web_sys::HtmlInputElement =
                ev.target().expect("event target").unchecked_into();
            let files = match input.files() {
                Some(f) => f,
                None => return,
            };
            let file = match files.get(0) {
                Some(f) => f,
                None => return,
            };

            let ctx = ensure_audio_context(&audio_ctx_holder);

            // Create AudioDeck on first load (requires AudioContext)
            {
                let mut deck_opt = audio_deck.borrow_mut();
                if deck_opt.is_none() {
                    *deck_opt = Some(AudioDeck::new(ctx.clone()));
                }
            }
            let deck_rc = audio_deck.borrow().as_ref().unwrap().clone();

            let state = state.clone();
            spawn_local(async move {
                load_audio_file(file, deck_rc, state, ctx).await;
            });
        }
    };

    let deck_class = format!("deck deck-{}", side.to_lowercase());

    view! {
        <div class=deck_class>
            <h2 class="deck-label">{format!("DECK {side}")}</h2>
            <TrackLabel state=state.clone()/>
            <input
                type="file"
                accept=".mp3,.wav,.ogg,.flac,.aac"
                style="display:none"
                node_ref=file_input
                on:change=on_file_change
            />
            <button class="btn-load" on:click=on_load_click>
                "Load Track"
            </button>
        </div>
    }
}

/// Displays the loaded track name and duration.
#[component]
pub fn TrackLabel(state: DeckState) -> impl IntoView {
    view! {
        <div class="track-label">
            <span class="track-name">
                {move || state.track_name.get()
                    .map(|name| truncate_name(&name, 24))
                    .unwrap_or_else(|| "— no track —".to_string())}
            </span>
            <span class="track-duration">
                {move || format_duration(state.duration_secs.get())}
            </span>
        </div>
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}…", &name[..max_len - 1])
    }
}

fn format_duration(secs: f64) -> String {
    if secs == 0.0 {
        return "--:--".to_string();
    }
    let total = secs as u64;
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::{format_duration, truncate_name};

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(0.0), "--:--");
    }

    #[test]
    fn test_format_duration_values() {
        assert_eq!(format_duration(125.0), "2:05");
        assert_eq!(format_duration(60.0), "1:00");
        assert_eq!(format_duration(3661.0), "61:01");
    }

    #[test]
    fn test_truncate_name_short() {
        assert_eq!(truncate_name("short.mp3", 24), "short.mp3");
    }

    #[test]
    fn test_truncate_name_long() {
        let long = "averylongtracknamefromadisk.mp3";
        let result = truncate_name(long, 24);
        assert!(result.ends_with('…'));
    }
}

