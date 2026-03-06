/// `requestAnimationFrame` loop — the engine that drives all canvas rendering
/// and keeps `DeckState.current_secs` in sync with the Web Audio clock.
///
/// Started once via `spawn_local` after `DeckView` mounts. Runs indefinitely.
use std::rc::Rc;
use std::cell::RefCell;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use leptos::task::spawn_local;
use leptos::html;
use leptos::prelude::*;
use crate::audio::deck_audio::{AudioDeck, read_vu_level};
use crate::state::DeckState;
use crate::canvas::waveform_draw::{draw_waveform, WaveformCache};
use crate::canvas::platter_draw::draw_platter;

/// Start the shared rAF loop.
///
/// Uses the classic recursive-closure pattern: an `Rc<RefCell<Option<Closure>>>`
/// that re-schedules itself. The outer `spawn_local` defers the first frame until
/// after the current synchronous render so all `NodeRef`s are populated.
#[allow(clippy::too_many_arguments)] // 8 params needed: 2 states, 2 audio, 2 waveform, 2 platter
pub fn start_raf_loop(
    state_a:       DeckState,
    state_b:       DeckState,
    audio_a:       Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    audio_b:       Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    waveform_a:    NodeRef<html::Canvas>,
    waveform_b:    NodeRef<html::Canvas>,
    platter_a:     NodeRef<html::Canvas>,
    platter_b:     NodeRef<html::Canvas>,
) {
    // Per-deck offscreen canvas caches — allocated once, shared into the closure.
    let cache_a = WaveformCache::new();
    let cache_b = WaveformCache::new();

    spawn_local(async move {
        #[allow(clippy::type_complexity)]
        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            // ── 1. Update current_secs from the audio clock ──────────────────
            update_current_time(&audio_a, &state_a);
            update_current_time(&audio_b, &state_b);

            // ── 2. Check loop boundaries ─────────────────────────────────────
            check_loop(&audio_a, &state_a);
            check_loop(&audio_b, &state_b);

            // ── 3. Update VU meter levels ─────────────────────────────────────
            update_vu_level(&audio_a, &state_a);
            update_vu_level(&audio_b, &state_b);

            // ── 4. Draw waveforms ─────────────────────────────────────────────
            draw_waveform(&waveform_a, &state_a, &cache_a, "a");
            draw_waveform(&waveform_b, &state_b, &cache_b, "b");

            // ── 5. Draw platters ──────────────────────────────────────────────
            draw_platter(&platter_a, &state_a, "a");
            draw_platter(&platter_b, &state_b, "b");

            // ── 6. Schedule the next frame ────────────────────────────────────
            web_sys::window()
                .expect("raf_loop — window is always present in a browser WASM context")
                .request_animation_frame(
                    f.borrow().as_ref()
                        .expect("RAF closure was assigned two lines above; this Option is always Some")
                        .as_ref().unchecked_ref(),
                )
                .expect("raf_loop — request_animation_frame is infallible with a valid Closure");
        }));

        // Kick off the first frame.
        web_sys::window()
            .expect("raf_loop — window is always present in a browser WASM context")
            .request_animation_frame(
                g.borrow().as_ref()
                    .expect("RAF closure was assigned in the block above; this Option is always Some")
                    .as_ref().unchecked_ref(),
            )
            .expect("raf_loop — first request_animation_frame is infallible with a valid Closure");
    });
}

// ── Per-frame helpers ─────────────────────────────────────────────────────────

/// Write the current playhead position from the audio clock into `DeckState.current_secs`.
fn update_current_time(
    audio_holder: &Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    state:        &DeckState,
) {
    if let Some(ref deck_rc) = *audio_holder.borrow() {
        let deck = deck_rc.borrow();
        // Only advance time when source is active (playing).
        if deck.source.is_some() {
            let pos = deck.current_position();
            state.current_secs.set(pos);
        }
    }
}

/// Write RMS VU level from the AnalyserNode into `DeckState.vu_level` (0.0–1.0).
fn update_vu_level(
    audio_holder: &Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    state:        &DeckState,
) {
    if let Some(ref deck_rc) = *audio_holder.borrow() {
        let level = read_vu_level(&deck_rc.borrow().analyser);
        state.vu_level.set(level);
    }
}
fn check_loop(
    audio_holder: &Rc<RefCell<Option<Rc<RefCell<AudioDeck>>>>>,
    state:        &DeckState,
) {
    if !state.loop_active.get_untracked() {
        return;
    }
    let current  = state.current_secs.get_untracked();
    let loop_out = state.loop_out.get_untracked();
    let loop_in  = state.loop_in.get_untracked();

    if loop_out <= loop_in || current < loop_out {
        return;
    }

    // Reached loop-out — seek back to loop-in and resume.
    if let Some(ref deck_rc) = *audio_holder.borrow() {
        let rate = state.playback_rate.get_untracked() as f32;
        deck_rc.borrow_mut().seek(loop_in, rate);
        state.current_secs.set(loop_in);
    }
}
