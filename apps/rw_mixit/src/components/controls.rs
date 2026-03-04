/// Transport controls component: Play/Pause, Stop, Cue, and Nudge (−/+) buttons.
use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::*;
use crate::audio::deck_audio::AudioDeck;
use crate::state::DeckState;

/// Playback transport buttons for one deck.
///
/// All button callbacks borrow `audio_deck_holder` to interact with the audio
/// engine. If no audio deck has been created yet (no file loaded) the callbacks
/// are no-ops.
#[component]
pub fn Controls(
    state:             DeckState,
    audio_deck_holder: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
) -> impl IntoView {
    // ── Play / Pause ─────────────────────────────────────────────────────────
    let on_play_pause = {
        let state = state.clone();
        let holder = audio_deck_holder.clone();
        move |_: web_sys::MouseEvent| {
            if let Some(ref deck_rc) = *holder.borrow() {
                let mut deck = deck_rc.borrow_mut();
                if deck.source.is_some() {
                    // Currently playing → pause.
                    let pos = deck.pause();
                    drop(deck);
                    state.is_playing.set(false);
                    state.current_secs.set(pos);
                } else {
                    // Not playing → play from current offset.
                    let offset = deck.offset_at_play;
                    let rate   = state.playback_rate.get_untracked() as f32;
                    deck.play(offset, rate);
                    drop(deck);
                    state.is_playing.set(true);
                }
            }
        }
    };

    // ── Stop ─────────────────────────────────────────────────────────────────
    let on_stop = {
        let state = state.clone();
        let holder = audio_deck_holder.clone();
        move |_: web_sys::MouseEvent| {
            if let Some(ref deck_rc) = *holder.borrow() {
                deck_rc.borrow_mut().stop();
            }
            state.is_playing.set(false);
            state.current_secs.set(0.0);
        }
    };

    // ── Cue ──────────────────────────────────────────────────────────────────
    let on_cue = {
        let state = state.clone();
        let holder = audio_deck_holder.clone();
        move |_: web_sys::MouseEvent| {
            if let Some(ref deck_rc) = *holder.borrow() {
                let rate = state.playback_rate.get_untracked() as f32;
                deck_rc.borrow_mut().cue(rate);
            }
        }
    };

    // ── Nudge − ──────────────────────────────────────────────────────────────
    let nudge_minus_start = {
        let holder = audio_deck_holder.clone();
        move |_: web_sys::MouseEvent| {
            if let Some(ref deck_rc) = *holder.borrow() {
                deck_rc.borrow_mut().nudge_start(-1.0);
            }
        }
    };
    let nudge_minus_end_rc = {
        let holder = audio_deck_holder.clone();
        Rc::new(move || {
            if let Some(ref deck_rc) = *holder.borrow() {
                deck_rc.borrow_mut().nudge_end();
            }
        })
    };
    let nudge_minus_end_up  = nudge_minus_end_rc.clone();
    let nudge_minus_end_out = nudge_minus_end_rc;

    // ── Nudge + ──────────────────────────────────────────────────────────────
    let nudge_plus_start = {
        let holder = audio_deck_holder.clone();
        move |_: web_sys::MouseEvent| {
            if let Some(ref deck_rc) = *holder.borrow() {
                deck_rc.borrow_mut().nudge_start(1.0);
            }
        }
    };
    let nudge_plus_end_rc = {
        let holder = audio_deck_holder.clone();
        Rc::new(move || {
            if let Some(ref deck_rc) = *holder.borrow() {
                deck_rc.borrow_mut().nudge_end();
            }
        })
    };
    let nudge_plus_end_up  = nudge_plus_end_rc.clone();
    let nudge_plus_end_out = nudge_plus_end_rc;

    view! {
        <div class="controls">
            <button
                class="btn-transport btn-play-pause"
                on:click=on_play_pause
            >
                {move || if state.is_playing.get() { "⏸" } else { "▶" }}
            </button>
            <button class="btn-transport btn-stop" on:click=on_stop>
                "■"
            </button>
            <button class="btn-transport btn-cue" on:click=on_cue>
                "CUE"
            </button>
            <button
                class="btn-nudge btn-nudge-minus"
                on:mousedown=nudge_minus_start
                on:mouseup=move |_| nudge_minus_end_up()
                on:mouseleave=move |_| nudge_minus_end_out()
            >
                "−"
            </button>
            <button
                class="btn-nudge btn-nudge-plus"
                on:mousedown=nudge_plus_start
                on:mouseup=move |_| nudge_plus_end_up()
                on:mouseleave=move |_| nudge_plus_end_out()
            >
                "+"
            </button>
        </div>
    }
}
