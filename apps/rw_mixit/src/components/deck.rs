/// Deck and DeckView components: the main three-column layout and individual deck UI.
use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;
use web_sys::AudioContext;

use crate::audio::{ensure_audio_context, deck_audio::AudioDeck};
use crate::audio::deck_audio::{apply_sweep_filter};
use crate::audio::loader::load_audio_file;
use crate::audio::bpm::sync_rate;
use crate::audio::mixer_audio::MixerAudio;
use crate::canvas::raf_loop::start_raf_loop;
use crate::canvas::platter_draw::PLATTER_SIZE;
use crate::components::controls::Controls;
use crate::components::eq::{EqKnobs, FilterKnob, VuMeter};
use crate::components::fx_panel::FxPanel;
use crate::components::hot_cues::HotCues;
use crate::components::loop_controls::LoopControls;
use crate::components::mixer::Mixer;
use crate::components::pitch_fader::PitchFader;
use crate::state::{DeckState, MixerState};
use crate::state::mixer::DeckId;
use crate::utils::keyboard::register_keyboard_shortcuts;

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

    // Shared mixer output nodes (xfade gains + master gain). Created lazily
    // on first file load when the AudioContext becomes available.
    let mixer_audio: Rc<RefCell<Option<MixerAudio>>> = Rc::new(RefCell::new(None));

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

    // M10 — Register global keyboard shortcuts and keep the listeners alive for
    // the entire application lifetime.  `std::mem::forget` prevents the
    // EventListeners from calling removeEventListener on drop — same pattern as
    // the hashchange listener in app.rs.
    let kb = register_keyboard_shortcuts(
        state_a.clone(),
        audio_a.clone(),
        state_b.clone(),
        audio_b.clone(),
    );
    std::mem::forget(kb);

    view! {
        <div class="deck-row">
            <Deck
                side="A"
                deck_id=DeckId::A
                state=state_a.clone()
                audio_ctx_holder=audio_ctx_holder.clone()
                audio_deck_holder=audio_a.clone()
                waveform_ref=waveform_a_ref
                platter_ref=platter_a_ref
                bpm_own=mixer_state.bpm_a
                bpm_other=mixer_state.bpm_b
                sync_master=mixer_state.sync_master
                mixer_audio=mixer_audio.clone()
                crossfader=mixer_state.crossfader
                master_volume=mixer_state.master_volume
            />
            <Mixer
                mixer_state=mixer_state.clone()
                mixer_audio=mixer_audio.clone()
                vol_a=state_a.volume
                vol_b=state_b.volume
            />
            <Deck
                side="B"
                deck_id=DeckId::B
                state=state_b.clone()
                audio_ctx_holder=audio_ctx_holder
                audio_deck_holder=audio_b.clone()
                waveform_ref=waveform_b_ref
                platter_ref=platter_b_ref
                bpm_own=mixer_state.bpm_b
                bpm_other=mixer_state.bpm_a
                sync_master=mixer_state.sync_master
                mixer_audio=mixer_audio.clone()
                crossfader=mixer_state.crossfader
                master_volume=mixer_state.master_volume
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
    /// Shared mixer output nodes — created on first load, then connected.
    mixer_audio:       Rc<RefCell<Option<MixerAudio>>>,
    /// Current crossfader value — applied when MixerAudio is first created.
    crossfader:        RwSignal<f32>,
    /// Current master volume — applied when MixerAudio is first created.
    master_volume:     RwSignal<f32>,
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
        let mixer_audio        = mixer_audio.clone();
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

            // T5.1 — Ensure MixerAudio exists and is wired to destination.
            // Both deck closures race here; only the first one creates it.
            {
                let mut ma_opt = mixer_audio.borrow_mut();
                if ma_opt.is_none() {
                    let ma = MixerAudio::new(&ctx);
                    // Apply current slider positions in case user moved them
                    // before loading any track (Effects won't re-fire otherwise).
                    let xf = crossfader.get_untracked();
                    let mv = master_volume.get_untracked();
                    ma.set_crossfader(xf);
                    ma.master_gain.gain().set_value(mv);
                    *ma_opt = Some(ma);
                }
            }

            // Create AudioDeck on first load and connect to mixer output.
            // MixerAudio is guaranteed to exist at this point (created above).
            // Both operations are guarded by the same `is_none()` check so the
            // audio graph wiring happens exactly once per deck lifetime.
            {
                let mut deck_opt = audio_deck_holder.borrow_mut();
                if deck_opt.is_none() {
                    let deck = AudioDeck::new(ctx.clone());
                    let ma_opt = mixer_audio.borrow();
                    if let Some(ref ma) = *ma_opt {
                        let xfade_gain = match deck_id {
                            DeckId::A => ma.xfade_gain_a.clone(),
                            DeckId::B => ma.xfade_gain_b.clone(),
                        };
                        deck.borrow().connect_to_mixer_output(&xfade_gain);
                    }
                    *deck_opt = Some(deck);
                }
            }
            let deck_rc = audio_deck_holder.borrow().as_ref()
                .expect("AudioDeck was just created in the is_none() block above")
                .clone();

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

    // T5.3 — Propagate volume signal changes to the channel_gain AudioParam.
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let vol = state_eff.volume.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().channel_gain.gain().set_value(vol);
            }
        });
    }

    // T8.2 — Propagate EQ signal changes to the BiquadFilterNode gains.
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let db = state_eff.eq_high.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().eq_high.gain().set_value(db);
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let db = state_eff.eq_mid.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().eq_mid.gain().set_value(db);
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let db = state_eff.eq_low.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().eq_low.gain().set_value(db);
            }
        });
    }

    // T8.3 — Propagate filter_val signal to the sweep BiquadFilterNode.
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let val = state_eff.filter_val.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                apply_sweep_filter(&deck_rc.borrow().sweep_filter, val);
            }
        });
    }

    // ── T9.1 — Echo toggle + param Effects ───────────────────────────────────
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let on = state_eff.fx_echo.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                if on { deck_rc.borrow().enable_echo(); }
                else  { deck_rc.borrow().disable_echo(); }
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let time = state_eff.fx_echo_time.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().echo_delay.delay_time().set_value(time);
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            // Hard-cap feedback at 0.85 to prevent runaway feedback loop.
            let fb = state_eff.fx_echo_feedback_val.get().min(0.85);
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().echo_feedback.gain().set_value(fb);
            }
        });
    }

    // ── T9.2 — Reverb toggle + IR param Effects ───────────────────────────────
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let on = state_eff.fx_reverb.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                if on { deck_rc.borrow().enable_reverb(); }
                else  { deck_rc.borrow().disable_reverb(); }
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let duration = state_eff.fx_reverb_duration.get();
            let decay    = state_eff.fx_reverb_decay.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().reload_reverb_ir(duration, decay);
            }
        });
    }

    // ── T9.3 — Flanger toggle + param Effects ─────────────────────────────────
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let on = state_eff.fx_flanger.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                if on { deck_rc.borrow().enable_flanger(); }
                else  { deck_rc.borrow().disable_flanger(); }
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let rate = state_eff.fx_flanger_rate.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().flanger_lfo.frequency().set_value(rate);
            }
        });
    }
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let depth = state_eff.fx_flanger_depth.get();
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                deck_rc.borrow().flanger_depth.gain().set_value(depth);
            }
        });
    }

    // ── T9.4 — Stutter toggle + subdivision Effect ────────────────────────────
    {
        let state_eff = state.clone();
        let holder_eff = audio_deck_holder.clone();
        Effect::new(move |_| {
            let on     = state_eff.fx_stutter.get();
            let subdiv = state_eff.fx_stutter_subdiv.get();
            let bpm    = bpm_own.get().unwrap_or(120.0);
            if let Some(ref deck_rc) = *holder_eff.borrow() {
                if on { deck_rc.borrow().enable_stutter(bpm, subdiv as f64); }
                else  { deck_rc.borrow().disable_stutter(); }
            }
        });
    }

    // ── T9.5 — Scratch pointer event handlers ─────────────────────────────────
    let on_platter_pointerdown = {
        let state_ptr  = state.clone();
        let holder_ptr = audio_deck_holder.clone();
        move |ev: web_sys::PointerEvent| {
            if !state_ptr.fx_scratch.get_untracked() { return; }
            let canvas: web_sys::HtmlCanvasElement =
                ev.target().expect("pointerdown target").unchecked_into();
            let cx    = canvas.width()  as f64 / 2.0;
            let cy    = canvas.height() as f64 / 2.0;
            let angle = (ev.offset_y() as f64 - cy)
                            .atan2(ev.offset_x() as f64 - cx);
            let now   = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);
            // Capture pointer so we keep receiving move/up events outside the canvas.
            let _ = canvas.unchecked_ref::<web_sys::Element>()
                .set_pointer_capture(ev.pointer_id());
            if let Some(ref deck_rc) = *holder_ptr.borrow() {
                deck_rc.borrow_mut().scratch_start(angle, now);
            }
        }
    };

    let on_platter_pointermove = {
        let state_ptr  = state.clone();
        let holder_ptr = audio_deck_holder.clone();
        move |ev: web_sys::PointerEvent| {
            if !state_ptr.fx_scratch.get_untracked() { return; }
            let canvas: web_sys::HtmlCanvasElement =
                ev.target().expect("pointermove target").unchecked_into();
            let cx    = canvas.width()  as f64 / 2.0;
            let cy    = canvas.height() as f64 / 2.0;
            let angle = (ev.offset_y() as f64 - cy)
                            .atan2(ev.offset_x() as f64 - cx);
            let now   = web_sys::window()
                .and_then(|w| w.performance())
                .map(|p| p.now())
                .unwrap_or(0.0);
            if let Some(ref deck_rc) = *holder_ptr.borrow() {
                deck_rc.borrow_mut().scratch_move(angle, now);
            }
        }
    };

    // Shared scratch-end logic for pointerup and pointerleave.
    let on_platter_scratch_end = {
        let holder_ptr = audio_deck_holder.clone();
        Rc::new(move || {
            if let Some(ref deck_rc) = *holder_ptr.borrow() {
                deck_rc.borrow_mut().scratch_end();
            }
        })
    };
    let on_platter_pointerup = {
        let end_fn = on_platter_scratch_end.clone();
        move |_: web_sys::PointerEvent| { end_fn(); }
    };
    let on_platter_pointerleave = {
        let end_fn = on_platter_scratch_end.clone();
        move |_: web_sys::PointerEvent| { end_fn(); }
    };

    view! {
        <div class=deck_class>
            <h2 class="deck-label">{format!("DECK {side}")}</h2>
            <TrackLabel state=state.clone()/>

            // Inline error message — shown when a file fails to load or decode.
            {move || state.load_error.get().map(|err| view! {
                <div class="load-error">{err}</div>
            })}

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
                on:pointerdown=on_platter_pointerdown
                on:pointermove=on_platter_pointermove
                on:pointerup=on_platter_pointerup
                on:pointerleave=on_platter_pointerleave
            />

            // Transport controls (T2.5)
            <Controls state=state.clone() audio_deck_holder=audio_deck_holder.clone()/>

            // Loop controls (T6.1–T6.3, T6.6)
            <LoopControls state=state.clone() bpm=bpm_own/>

            // Hot cue buttons (T7.1–T7.4)
            <HotCues state=state.clone() audio_deck_holder=audio_deck_holder.clone()/>

            // EQ knobs (T8.1–T8.2) + Filter knob (T8.3) + VU meter (T8.5)
            <div class="eq-filter-row">
                <EqKnobs state=state.clone()/>
                <FilterKnob state=state.clone()/>
                <VuMeter state=state.clone()/>
            </div>

            // FX Panel: Echo, Reverb, Flanger, Stutter, Scratch (T9.1–T9.6)
            <FxPanel state=state.clone()/>

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
            state.zoom_level.set(state.zoom_level.get_untracked().zoom_out());
        }
    };
    let on_zoom_in = {
        let state = state.clone();
        move |_: web_sys::MouseEvent| {
            state.zoom_level.set(state.zoom_level.get_untracked().zoom_in());
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
                    if let Some(new_rate) = sync_rate(playback_rate.get_untracked(), own_bpm, other_bpm) {
                        playback_rate.set(new_rate);
                        // The deck we're syncing TO becomes the master reference
                        let other_id = match deck_id {
                            DeckId::A => DeckId::B,
                            DeckId::B => DeckId::A,
                        };
                        sync_master.set(Some(other_id));
                    }
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
