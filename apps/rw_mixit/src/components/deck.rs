/// Deck and DeckView components: the main three-column layout and individual deck UI.
use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::AudioContext;

use crate::audio::{ensure_audio_context, deck_audio::AudioDeck};
use crate::audio::loader::load_audio_file;
use crate::canvas::raf_loop::start_raf_loop;
use crate::canvas::platter_draw::PLATTER_SIZE;
use crate::components::controls::Controls;
use crate::components::mixer::Mixer;
use crate::components::pitch_fader::PitchFader;
use crate::state::DeckState;

/// Waveform canvas dimensions (pixels).
const WAVEFORM_WIDTH:  u32 = 600;
const WAVEFORM_HEIGHT: u32 = 80;

/// Three-column layout: `[Deck A] [Mixer] [Deck B]`.
///
/// Creates both deck states and lazily-initialised audio deck holders, wires
/// NodeRefs for the waveform canvases, and starts the shared rAF loop.
#[component]
pub fn DeckView() -> impl IntoView {
    // Shared AudioContext — both decks use the same one for accurate sync.
    let audio_ctx_holder: Rc<RefCell<Option<AudioContext>>> = Rc::new(RefCell::new(None));

    // Per-deck reactive state.
    let state_a = DeckState::new();
    let state_b = DeckState::new();

    // Per-deck audio graph holders (None until first file load).
    let audio_a: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>> = Rc::new(RefCell::new(None));
    let audio_b: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>> = Rc::new(RefCell::new(None));

    // Canvas NodeRefs for the waveform draw pass.
    let waveform_a_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();
    let waveform_b_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();

    // Canvas NodeRefs for the platter draw pass.
    let platter_a_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();
    let platter_b_ref: NodeRef<leptos::html::Canvas> = NodeRef::new();

    // Start the rAF loop (deferred via spawn_local so NodeRefs are populated).
    start_raf_loop(
        state_a.clone(),
        state_b.clone(),
        audio_a.clone(),
        audio_b.clone(),
        waveform_a_ref,
        waveform_b_ref,
        platter_a_ref,
        platter_b_ref,
    );

    view! {
        <div class="deck-row">
            <Deck
                side="A"
                state=state_a
                audio_ctx_holder=audio_ctx_holder.clone()
                audio_deck_holder=audio_a
                waveform_ref=waveform_a_ref
                platter_ref=platter_a_ref
            />
            <Mixer/>
            <Deck
                side="B"
                state=state_b
                audio_ctx_holder=audio_ctx_holder
                audio_deck_holder=audio_b
                waveform_ref=waveform_b_ref
                platter_ref=platter_b_ref
            />
        </div>
    }
}

/// A single DJ deck column.
///
/// Contains the track label, waveform canvas, platter canvas, transport
/// controls, pitch fader, and the hidden file input triggered by "Load Track".
#[component]
pub fn Deck(
    side:              &'static str,
    state:             DeckState,
    audio_ctx_holder:  Rc<RefCell<Option<AudioContext>>>,
    audio_deck_holder: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    waveform_ref:      NodeRef<leptos::html::Canvas>,
    platter_ref:       NodeRef<leptos::html::Canvas>,
) -> impl IntoView {
    let file_input: NodeRef<leptos::html::Input> = NodeRef::new();

    let on_load_click = {
        move |_: web_sys::MouseEvent| {
            if let Some(input) = file_input.get() {
                input.click();
            }
        }
    };

    let on_file_change = {
        let audio_ctx_holder   = audio_ctx_holder.clone();
        let audio_deck_holder  = audio_deck_holder.clone();
        let state              = state.clone();
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

            // Create AudioDeck on first load (requires AudioContext).
            {
                let mut deck_opt = audio_deck_holder.borrow_mut();
                if deck_opt.is_none() {
                    *deck_opt = Some(AudioDeck::new(ctx.clone()));
                }
            }
            let deck_rc = audio_deck_holder.borrow().as_ref().unwrap().clone();

            let state = state.clone();
            spawn_local(async move {
                load_audio_file(file, deck_rc, state, ctx).await;
            });
        }
    };

    // Waveform seek on click: compute click position → time → seek.
    let on_waveform_click = {
        let state             = state.clone();
        let audio_deck_holder = audio_deck_holder.clone();
        move |ev: web_sys::MouseEvent| {
            // Only seek when not playing (cue mode / paused).
            if state.is_playing.get_untracked() {
                return;
            }
            let duration = state.duration_secs.get_untracked();
            if duration <= 0.0 {
                return;
            }
            let canvas_el = waveform_ref.get_untracked();
            let canvas_el = match canvas_el {
                Some(c) => c,
                None => return,
            };
            let canvas_width = canvas_el.width() as f64;
            let click_x = ev.offset_x() as f64;
            // The waveform is scrolled so the playhead is at the center.
            // Clicking at click_x relative to center maps to a time delta.
            let center_x    = canvas_width / 2.0;
            let current     = state.current_secs.get_untracked();
            let secs_per_px = if canvas_width > 0.0 { duration / canvas_width } else { 0.0 };
            let seek_pos = (current + (click_x - center_x) * secs_per_px).clamp(0.0, duration);

            if let Some(ref deck_rc) = *audio_deck_holder.borrow() {
                let rate = state.playback_rate.get_untracked() as f32;
                deck_rc.borrow_mut().seek(seek_pos, rate);
                state.current_secs.set(seek_pos);
            }
        }
    };

    let deck_class = format!("deck deck-{}", side.to_lowercase());

    // T3.5 — Propagate playback_rate signal changes to the live AudioParam.
    // Fires once on mount (source is None → no-op) and again whenever the
    // pitch fader (or any other writer) changes `state.playback_rate`.
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let rate = state_eff.playback_rate.get() as f32;
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                if let Some(ref src) = deck_rc.borrow().source {
                    src.playback_rate().set_value(rate);
                }
            }
        });
    }

    view! {
        <div class=deck_class>
            <h2 class="deck-label">{format!("DECK {side}")}</h2>
            <TrackLabel state=state.clone()/>

            // Waveform canvas
            <canvas
                class="waveform-canvas"
                width=WAVEFORM_WIDTH
                height=WAVEFORM_HEIGHT
                node_ref=waveform_ref
                on:click=on_waveform_click
            />

            // Zoom controls (T2.11)
            <ZoomControls state=state.clone()/>

            // Platter canvas (T3.1–T3.3 / T3.6)
            <canvas
                class="platter-canvas"
                width=PLATTER_SIZE
                height=PLATTER_SIZE
                node_ref=platter_ref
            />

            // Transport controls (T2.5)
            <Controls state=state.clone() audio_deck_holder=audio_deck_holder.clone()/>

            // Pitch fader (T3.4)
            <PitchFader state=state.clone()/>

            // Hidden file input
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

/// Waveform zoom controls: [−] decreases zoom, [+] increases (1× → 8×).
#[component]
pub fn ZoomControls(state: DeckState) -> impl IntoView {
    let on_zoom_out = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            let current = state.zoom_level.get_untracked();
            if current > 1 {
                state.zoom_level.set(current / 2);
            }
        }
    };
    let on_zoom_in = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            let current = state.zoom_level.get_untracked();
            if current < 8 {
                state.zoom_level.set(current * 2);
            }
        }
    };

    view! {
        <div class="zoom-controls">
            <button class="btn-zoom" on:click=on_zoom_out>"−"</button>
            <span class="zoom-label">
                {move || format!("{}×", state.zoom_level.get())}
            </span>
            <button class="btn-zoom" on:click=on_zoom_in>"+"</button>
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
    let total   = secs as u64;
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
