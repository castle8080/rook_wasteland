use leptos::prelude::*;

use crate::state::{DeckAContext, DeckBContext, MixerState};

/// Settings view: per-deck reverb parameters and crossfader curve selector.
///
/// Reads `DeckAContext`, `DeckBContext`, and `MixerState` from Leptos context
/// (provided by `DeckView`).  Writing any signal here triggers the corresponding
/// `Effect` inside the mounted-but-hidden `DeckView`, so audio changes are live.
#[component]
pub fn SettingsView() -> impl IntoView {
    // Fetch contexts — DeckView is always mounted so these are always present.
    let deck_a = use_context::<DeckAContext>()
        .expect("SettingsView: DeckAContext not provided — is DeckView mounted?");
    let deck_b = use_context::<DeckBContext>()
        .expect("SettingsView: DeckBContext not provided — is DeckView mounted?");
    let mixer = use_context::<MixerState>()
        .expect("SettingsView: MixerState not provided — is DeckView mounted?");

    let state_a = deck_a.0;
    let state_b = deck_b.0;

    view! {
        <main class="settings-view">
            <section class="settings-card">
                <h1 class="settings-title">"Settings"</h1>

                // ── Crossfader curve ──────────────────────────────────────────
                <div class="settings-section">
                    <h2 class="settings-section-title">"Crossfader Curve"</h2>
                    <div class="settings-toggle-row">
                        <label class="settings-label">"Equal-Power (default)"</label>
                        <input
                            type="radio"
                            name="xfader-curve"
                            value="eq-power"
                            prop:checked=move || !mixer.crossfader_curve_linear.get()
                            on:change=move |_| mixer.crossfader_curve_linear.set(false)
                        />
                        <label class="settings-label">"Linear"</label>
                        <input
                            type="radio"
                            name="xfader-curve"
                            value="linear"
                            prop:checked=move || mixer.crossfader_curve_linear.get()
                            on:change=move |_| mixer.crossfader_curve_linear.set(true)
                        />
                    </div>
                </div>

                // ── Deck A reverb ─────────────────────────────────────────────
                <div class="settings-section">
                    <h2 class="settings-section-title">"Deck A — Reverb"</h2>

                    <div class="settings-row">
                        <label class="settings-label">"Duration (s)"</label>
                        <input
                            type="range"
                            class="settings-slider"
                            min="0.5" max="3.5" step="0.1"
                            prop:value=move || state_a.fx_reverb_duration.get().to_string()
                            on:input=move |ev| {
                                let val: f32 = event_target_value(&ev)
                                    .parse()
                                    .unwrap_or(1.2);
                                state_a.fx_reverb_duration.set(val);
                            }
                        />
                        <span class="settings-value">
                            {move || format!("{:.1}s", state_a.fx_reverb_duration.get())}
                        </span>
                    </div>

                    <div class="settings-row">
                        <label class="settings-label">"Decay"</label>
                        <input
                            type="range"
                            class="settings-slider"
                            min="0.5" max="4.0" step="0.1"
                            prop:value=move || state_a.fx_reverb_decay.get().to_string()
                            on:input=move |ev| {
                                let val: f32 = event_target_value(&ev)
                                    .parse()
                                    .unwrap_or(2.5);
                                state_a.fx_reverb_decay.set(val);
                            }
                        />
                        <span class="settings-value">
                            {move || format!("{:.1}", state_a.fx_reverb_decay.get())}
                        </span>
                    </div>
                </div>

                // ── Deck B reverb ─────────────────────────────────────────────
                <div class="settings-section">
                    <h2 class="settings-section-title">"Deck B — Reverb"</h2>

                    <div class="settings-row">
                        <label class="settings-label">"Duration (s)"</label>
                        <input
                            type="range"
                            class="settings-slider"
                            min="0.5" max="3.5" step="0.1"
                            prop:value=move || state_b.fx_reverb_duration.get().to_string()
                            on:input=move |ev| {
                                let val: f32 = event_target_value(&ev)
                                    .parse()
                                    .unwrap_or(1.2);
                                state_b.fx_reverb_duration.set(val);
                            }
                        />
                        <span class="settings-value">
                            {move || format!("{:.1}s", state_b.fx_reverb_duration.get())}
                        </span>
                    </div>

                    <div class="settings-row">
                        <label class="settings-label">"Decay"</label>
                        <input
                            type="range"
                            class="settings-slider"
                            min="0.5" max="4.0" step="0.1"
                            prop:value=move || state_b.fx_reverb_decay.get().to_string()
                            on:input=move |ev| {
                                let val: f32 = event_target_value(&ev)
                                    .parse()
                                    .unwrap_or(2.5);
                                state_b.fx_reverb_decay.set(val);
                            }
                        />
                        <span class="settings-value">
                            {move || format!("{:.1}", state_b.fx_reverb_decay.get())}
                        </span>
                    </div>
                </div>
            </section>
        </main>
    }
}
