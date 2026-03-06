/// EQ knobs, sweep-filter knob, and VU meter components (Milestone 8).
///
/// - `EqKnobs`   — 3-band EQ: High (+8 kHz shelf), Mid (1 kHz peak), Low (200 Hz shelf).
/// - `FilterKnob` — Sweep filter: left = low-pass closed, center = flat, right = high-pass.
/// - `VuMeter`    — Vertical bar driven reactively by `DeckState.vu_level` (0.0–1.0).
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::state::DeckState;

/// Three-band EQ panel (High / Mid / Low) in knob layout.
///
/// Each band maps to a `BiquadFilterNode.gain` updated via `Effect` in the `Deck` component.
#[component]
pub fn EqKnobs(state: DeckState) -> impl IntoView {
    view! {
        <div class="eq-panel">
            <span class="eq-panel-label">"EQ"</span>
            <div class="eq-knobs">
                <EqKnob label="HI"  value=state.eq_high />
                <EqKnob label="MID" value=state.eq_mid  />
                <EqKnob label="LOW" value=state.eq_low  />
            </div>
        </div>
    }
}

/// A single EQ rotary knob: label + vertical range input (−12 to +12 dB) + readout.
#[component]
fn EqKnob(label: &'static str, value: RwSignal<f32>) -> impl IntoView {
    let on_input = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = ev
            .target()
            .expect("EqKnob — event target")
            .unchecked_into();
        if let Ok(v) = input.value().parse::<f32>() {
            value.set(v);
        }
    };

    view! {
        <div class="eq-knob-group">
            <label class="eq-knob-label">{label}</label>
            <input
                type="range"
                class="knob eq-knob"
                min="-12"
                max="12"
                step="0.5"
                prop:value=move || value.get().to_string()
                on:input=on_input
            />
            <span class="eq-knob-readout">
                {move || {
                    let v = value.get();
                    if v == 0.0 { "0 dB".to_string() }
                    else if v > 0.0 { format!("+{v:.0}") }
                    else { format!("{v:.0}") }
                }}
            </span>
        </div>
    }
}

/// Sweep filter knob: −1.0 (low-pass closed) → 0.0 (flat) → +1.0 (high-pass closed).
///
/// The `filter_val` signal drives `apply_sweep_filter` via `Effect` in the `Deck` component.
#[component]
pub fn FilterKnob(state: DeckState) -> impl IntoView {
    let on_input = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = ev
            .target()
            .expect("FilterKnob — event target")
            .unchecked_into();
        if let Ok(v) = input.value().parse::<f32>() {
            state.filter_val.set(v);
        }
    };

    view! {
        <div class="eq-panel filter-panel">
            <span class="eq-panel-label">"FILTER"</span>
            <div class="eq-knob-group">
                <label class="eq-knob-label">"LP ◄ ► HP"</label>
                <input
                    type="range"
                    class="filter-knob"
                    min="-1.0"
                    max="1.0"
                    step="0.01"
                    prop:value=move || state.filter_val.get().to_string()
                    on:input=on_input
                />
                <span class="eq-knob-readout">
                    {move || {
                        let v = state.filter_val.get();
                        if v.abs() < 0.02 { "FLAT".to_string() }
                        else if v < 0.0 { format!("LP {:.0}%", (-v) * 100.0) }
                        else { format!("HP {:.0}%", v * 100.0) }
                    }}
                </span>
            </div>
        </div>
    }
}

/// VU meter: a vertical bar whose `height` CSS property is driven reactively by
/// `DeckState.vu_level` (0.0–1.0, updated at ~60 fps by the rAF loop).
///
/// Styled green → yellow → red bottom-to-top via a CSS gradient.
#[component]
pub fn VuMeter(state: DeckState) -> impl IntoView {
    view! {
        <div class="vu-meter-wrapper">
            <div class="vu-meter-bar-container">
                <div
                    class="vu-meter-bar"
                    style=move || {
                        let level = state.vu_level.get().clamp(0.0, 1.0);
                        format!("height: {:.1}%", level * 100.0)
                    }
                />
            </div>
            <span class="vu-meter-label">"VU"</span>
        </div>
    }
}

