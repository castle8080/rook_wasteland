/// FX Panel component — Echo, Reverb, Flanger, Stutter, Scratch toggles with
/// per-effect parameter knobs (Milestone 9, T9.1–T9.6).
///
/// Five toggle buttons appear in a row. When an effect is active its button is
/// highlighted and an inline row of parameter knobs is shown below.
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::state::DeckState;

/// The full FX panel: toggle row + active-effect params.
#[component]
pub fn FxPanel(state: DeckState) -> impl IntoView {
    let state_echo_check = state.clone();
    let state_echo_params = state.clone();
    let state_reverb_check = state.clone();
    let state_reverb_params = state.clone();
    let state_flanger_check = state.clone();
    let state_flanger_params = state.clone();
    let state_stutter_check = state.clone();
    let state_stutter_params = state.clone();
    view! {
        <div class="fx-panel">
            <span class="fx-panel-label">"FX"</span>
            <div class="fx-toggle-row">
                <FxToggle label="ECHO"    active=state.fx_echo    />
                <FxToggle label="REVERB"  active=state.fx_reverb  />
                <FxToggle label="FLANGE"  active=state.fx_flanger />
                <FxToggle label="STUTTER" active=state.fx_stutter />
                <FxToggle label="SCRATCH" active=state.fx_scratch />
            </div>
            // Per-effect param rows — only visible when the effect is active.
            {move || state_echo_check.fx_echo.get().then(|| view! {
                <EchoParams state=state_echo_params.clone() />
            })}
            {move || state_reverb_check.fx_reverb.get().then(|| view! {
                <ReverbParams state=state_reverb_params.clone() />
            })}
            {move || state_flanger_check.fx_flanger.get().then(|| view! {
                <FlangerParams state=state_flanger_params.clone() />
            })}
            {move || state_stutter_check.fx_stutter.get().then(|| view! {
                <StutterParams state=state_stutter_params.clone() />
            })}
        </div>
    }
}

/// A single FX toggle button: grey when off, glowing when on.
#[component]
fn FxToggle(
    label:  &'static str,
    active: RwSignal<bool>,
) -> impl IntoView {
    let on_click = move |_: web_sys::MouseEvent| {
        active.update(|v| *v = !*v);
    };
    view! {
        <button
            class="btn-fx"
            class:btn-fx-active=move || active.get()
            on:click=on_click
        >
            {label}
        </button>
    }
}

// ── Parameter knob helper ────────────────────────────────────────────────────

/// A labelled horizontal range knob for a single FX parameter.
#[component]
fn FxKnob(
    label: &'static str,
    signal: RwSignal<f32>,
    min:    f64,
    max:    f64,
    step:   f64,
) -> impl IntoView {
    let on_input = move |ev: web_sys::Event| {
        let input: web_sys::HtmlInputElement = ev
            .target()
            .expect("FxKnob — event target")
            .unchecked_into();
        if let Ok(v) = input.value().parse::<f32>() {
            signal.set(v);
        }
    };

    view! {
        <div class="fx-knob-group">
            <label class="fx-knob-label">{label}</label>
            <input
                type="range"
                class="fx-knob"
                min=min.to_string()
                max=max.to_string()
                step=step.to_string()
                prop:value=move || signal.get().to_string()
                on:input=on_input
            />
            <span class="fx-knob-readout">
                {move || format!("{:.2}", signal.get())}
            </span>
        </div>
    }
}

// ── Echo params ──────────────────────────────────────────────────────────────

#[component]
fn EchoParams(state: DeckState) -> impl IntoView {
    view! {
        <div class="fx-params-row">
            <FxKnob
                label="TIME"
                signal=state.fx_echo_time
                min=0.05 max=2.0 step=0.05
            />
            <FxKnob
                label="FDBK"
                signal=state.fx_echo_feedback_val
                min=0.0 max=0.85 step=0.05
            />
        </div>
    }
}

// ── Reverb params ─────────────────────────────────────────────────────────────

#[component]
fn ReverbParams(state: DeckState) -> impl IntoView {
    view! {
        <div class="fx-params-row">
            <FxKnob
                label="DUR"
                signal=state.fx_reverb_duration
                min=0.5 max=3.5 step=0.1
            />
            <FxKnob
                label="DECAY"
                signal=state.fx_reverb_decay
                min=0.5 max=4.0 step=0.1
            />
        </div>
    }
}

// ── Flanger params ────────────────────────────────────────────────────────────

#[component]
fn FlangerParams(state: DeckState) -> impl IntoView {
    view! {
        <div class="fx-params-row">
            <FxKnob
                label="RATE"
                signal=state.fx_flanger_rate
                min=0.1 max=2.0 step=0.1
            />
            <FxKnob
                label="DEPTH"
                signal=state.fx_flanger_depth
                min=0.0 max=0.008 step=0.0005
            />
        </div>
    }
}

// ── Stutter params ────────────────────────────────────────────────────────────

/// Stutter subdivision selector: tap one of four buttons (1/4, 1/8, 1/16, 1/32).
#[component]
fn StutterParams(state: DeckState) -> impl IntoView {
    let subdivisions: &'static [(f32, &'static str)] =
        &[(4.0, "1/4"), (8.0, "1/8"), (16.0, "1/16"), (32.0, "1/32")];

    view! {
        <div class="fx-params-row fx-stutter-subdivs">
            <span class="fx-knob-label">"DIV"</span>
            {subdivisions.iter().map(|&(val, label)| {
                let subdiv_sig = state.fx_stutter_subdiv;
                let on_click = move |_: web_sys::MouseEvent| {
                    subdiv_sig.set(val);
                };
                view! {
                    <button
                        class="btn-fx btn-subdiv"
                        class:btn-fx-active=move || (subdiv_sig.get() - val).abs() < 0.01
                        on:click=on_click
                    >
                        {label}
                    </button>
                }
            }).collect_view()}
        </div>
    }
}
