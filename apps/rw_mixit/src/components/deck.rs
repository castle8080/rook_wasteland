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
use crate::state::{DeckState, MixerState};
use crate::state::mixer::DeckId;

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

    // Mixer state holds BPM signals and sync master (T4.3–T4.6).
    let mixer_state = MixerState::new();

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
                deck_id=DeckId::A
                state=state_a
                audio_ctx_holder=audio_ctx_holder.clone()
                audio_deck_holder=audio_a
                waveform_ref=waveform_a_ref
                platter_ref=platter_a_ref
                bpm_own=mixer_state.bpm_a
                bpm_other=mixer_state.bpm_b
                sync_master=mixer_state.sync_master
            />
            <Mixer/>
            <Deck
                side="B"
                deck_id=DeckId::B
                state=state_b
                audio_ctx_holder=audio_ctx_holder
                audio_deck_holder=audio_b
                waveform_ref=waveform_b_ref
                platter_ref=platter_b_ref
                bpm_own=mixer_state.bpm_b
                bpm_other=mixer_state.bpm_a
                sync_master=mixer_state.sync_master
            />
        </div>
    }
}

/// A single DJ deck column.
///
/// Contains the track label, waveform canvas, platter canvas, transport
/// controls, pitch fader, BPM panel, and the hidden file input triggered by "Load Track".
#[component]
pub fn Deck(
    side:              &'static str,
    deck_id:           DeckId,
    state:             DeckState,
    audio_ctx_holder:  Rc<RefCell<Option<AudioContext>>>,
    audio_deck_holder: Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    waveform_ref:      NodeRef<leptos::html::Canvas>,
    platter_ref:       NodeRef<leptos::html::Canvas>,
    bpm_own:           RwSignal<Option<f64>>,
    bpm_other:         RwSignal<Option<f64>>,
    sync_master:       RwSignal<Option<DeckId>>,
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
                load_audio_file(file, deck_rc, state, ctx, bpm_own).await;
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

            // BPM display, TAP, SYNC, MASTER (T4.4–T4.6)
            <BpmPanel
                deck_id=deck_id
                bpm_own=bpm_own
                bpm_other=bpm_other
                playback_rate=state.playback_rate
                sync_master=sync_master
            />

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

/// BPM display, TAP BPM, SYNC, and MASTER controls for one deck (T4.4–T4.6).
///
/// - Shows `bpm_own` formatted to one decimal; "---" when None.
/// - TAP BPM: records `performance.now()` timestamps, computes average from
///   a rolling window of the last 8 intervals, writes the result to `bpm_own`.
/// - SYNC: snaps this deck's playback rate so its BPM matches the other deck.
///   Formula: `new_rate = current_rate × (bpm_other / bpm_own)`.
/// - MASTER: clicking marks this deck as the tempo master in `sync_master`.
#[component]
pub fn BpmPanel(
    deck_id:       DeckId,
    bpm_own:       RwSignal<Option<f64>>,
    bpm_other:     RwSignal<Option<f64>>,
    playback_rate: RwSignal<f64>,
    sync_master:   RwSignal<Option<DeckId>>,
) -> impl IntoView {
    // Timestamps (ms) of recent taps; capped at 9 (= 8 intervals).
    let tap_times: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));

    let on_tap = {
        let tap_times = tap_times.clone();
        move |_: web_sys::MouseEvent| {
            let now = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);

            let mut taps = tap_times.borrow_mut();
            taps.push(now);
            if taps.len() > 9 {
                taps.remove(0);
            }
            if taps.len() >= 2 {
                let intervals: Vec<f64> = taps.windows(2).map(|w| w[1] - w[0]).collect();
                if let Some(bpm) = crate::audio::bpm::tap_bpm_from_intervals(&intervals) {
                    bpm_own.set(Some(bpm));
                }
            }
        }
    };

    let on_sync = {
        move |_: web_sys::MouseEvent| {
            let own = bpm_own.get_untracked();
            let other = bpm_other.get_untracked();
            if let (Some(own_bpm), Some(other_bpm)) = (own, other) {
                if own_bpm > 0.0 {
                    let new_rate = (playback_rate.get_untracked() * (other_bpm / own_bpm))
                        .clamp(0.25, 4.0);
                    playback_rate.set(new_rate);
                    sync_master.set(Some(deck_id));
                }
            }
        }
    };

    let on_set_master = {
        move |_: web_sys::MouseEvent| {
            sync_master.set(Some(deck_id));
        }
    };

    view! {
        <div class="bpm-panel">
            <div class="bpm-display">
                <span class="bpm-label">"BPM"</span>
                <span class="bpm-value">
                    {move || bpm_own.get()
                        .map(|b| format!("{b:.1}"))
                        .unwrap_or_else(|| "---".to_string())}
                </span>
                <button
                    class="btn-master"
                    class:master-active=move || sync_master.get() == Some(deck_id)
                    on:click=on_set_master
                >
                    "MASTER"
                </button>
            </div>
            <div class="bpm-controls">
                <button class="btn-tap" on:click=on_tap>"TAP"</button>
                <button class="btn-sync" on:click=on_sync>"SYNC"</button>
            </div>
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
