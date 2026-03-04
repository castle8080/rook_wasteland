use std::rc::Rc;
use std::cell::RefCell;
use leptos::prelude::*;

use crate::audio::mixer_audio::MixerAudio;
use crate::state::MixerState;

/// Central mixer panel — T5.2 through T5.5.
///
/// Layout (top → bottom):
///   - Channel faders A and B side by side (drive `DeckState.volume` signals)
///   - Crossfader (equal-power blend of both decks)
///   - Master volume knob
///
/// Audio routing Effects live here for the shared mixer nodes (crossfader +
/// master gain).  Per-deck channel gain is driven by an Effect in each
/// `Deck` component that watches `DeckState.volume`.
#[component]
pub fn Mixer(
    mixer_state: MixerState,
    mixer_audio: Rc<RefCell<Option<MixerAudio>>>,
    /// Channel-fader signal for Deck A — updated by the left fader slider.
    vol_a: RwSignal<f32>,
    /// Channel-fader signal for Deck B — updated by the right fader slider.
    vol_b: RwSignal<f32>,
) -> impl IntoView {

    // T5.2 — Crossfader Effect: equal-power cos/sin curve.
    {
        let mixer_audio = mixer_audio.clone();
        let crossfader  = mixer_state.crossfader;
        Effect::new(move |_| {
            let val = crossfader.get();
            if let Some(ref ma) = *mixer_audio.borrow() {
                ma.set_crossfader(val);
            }
        });
    }

    // T5.4 — Master volume Effect.
    {
        let mixer_audio  = mixer_audio.clone();
        let master_volume = mixer_state.master_volume;
        Effect::new(move |_| {
            let vol = master_volume.get();
            if let Some(ref ma) = *mixer_audio.borrow() {
                ma.master_gain.gain().set_value(vol);
            }
        });
    }

    view! {
        <div class="mixer">
            <p class="mixer-label">"MIXER"</p>

            // T5.3 — Channel faders
            <div class="mixer-channel-faders">
                <div class="channel-fader-group">
                    <span class="fader-deck-label fader-deck-a">"A"</span>
                    <input
                        type="range"
                        class="channel-fader"
                        min="0" max="1" step="0.01"
                        prop:value=move || vol_a.get().to_string()
                        on:input=move |ev| {
                            let val: f32 = event_target_value(&ev)
                                .parse()
                                .unwrap_or(1.0);
                            vol_a.set(val);
                        }
                    />
                    <span class="fader-value">
                        {move || format!("{:.0}%", vol_a.get() * 100.0)}
                    </span>
                </div>
                <div class="channel-fader-group">
                    <span class="fader-deck-label fader-deck-b">"B"</span>
                    <input
                        type="range"
                        class="channel-fader"
                        min="0" max="1" step="0.01"
                        prop:value=move || vol_b.get().to_string()
                        on:input=move |ev| {
                            let val: f32 = event_target_value(&ev)
                                .parse()
                                .unwrap_or(1.0);
                            vol_b.set(val);
                        }
                    />
                    <span class="fader-value">
                        {move || format!("{:.0}%", vol_b.get() * 100.0)}
                    </span>
                </div>
            </div>

            // T5.2 — Crossfader
            <div class="mixer-section">
                <span class="mixer-section-label">"CROSSFADER"</span>
                <div class="crossfader-track">
                    <span class="crossfader-end">"A"</span>
                    <input
                        type="range"
                        class="crossfader"
                        min="0" max="1" step="0.001"
                        prop:value=move || mixer_state.crossfader.get().to_string()
                        on:input=move |ev| {
                            let val: f32 = event_target_value(&ev)
                                .parse()
                                .unwrap_or(0.5);
                            mixer_state.crossfader.set(val);
                        }
                    />
                    <span class="crossfader-end">"B"</span>
                </div>
            </div>

            // T5.4 — Master volume
            <div class="mixer-section">
                <span class="mixer-section-label">"MASTER"</span>
                <div class="master-vol-row">
                    <input
                        type="range"
                        class="master-vol"
                        min="0" max="1" step="0.01"
                        prop:value=move || mixer_state.master_volume.get().to_string()
                        on:input=move |ev| {
                            let val: f32 = event_target_value(&ev)
                                .parse()
                                .unwrap_or(1.0);
                            mixer_state.master_volume.set(val);
                        }
                    />
                    <span class="master-vol-value">
                        {move || format!("{:.0}%", mixer_state.master_volume.get() * 100.0)}
                    </span>
                </div>
            </div>
        </div>
    }
}
