use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::export_menu::ExportMenu;
use crate::state::{randomize, AppState, KaleidoscopeParams};

/// Macro to reduce the boilerplate of a float range slider row.
///
/// Generates a `<div class="control-row">` containing a label, an
/// `<input type="range">` wired to an `RwSignal<f32>`, and a live
/// value display.
macro_rules! float_slider {
    (
        label   = $label:expr,
        signal  = $signal:expr,
        min     = $min:expr,
        max     = $max:expr,
        step    = $step:expr,
        fmt     = $fmt:expr
    ) => {{
        let sig = $signal;
        view! {
            <div class="control-row">
                <label class="control-label">{$label}</label>
                <input
                    type="range"
                    min={$min}
                    max={$max}
                    step={$step}
                    prop:value=move || sig.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: f32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or_default();
                        sig.set(val.clamp($min, $max));
                    }
                />
                <span class="control-value">
                    {move || format!($fmt, sig.get())}
                </span>
            </div>
        }
    }};
}

/// Side panel containing all kaleidoscope controls.
///
/// **Symmetry controls** (M4):
/// - **Segments** — integer slider (2–10); controls mirror count.
/// - **Rotation** — float slider (0–360°); rotates the whole pattern.
/// - **Zoom** — float slider (0.1–4.0); scales the source sampling radius.
/// - **Center** — read-only display; updated by dragging directly on the canvas.
///
/// **Visual effects** (M5):
/// - **Spiral** — float slider (0–1); vortex/spiral twist.
/// - **Ripple** — float slider (0–1); angular wave distortion.
/// - **Lens** — float slider (0–1); barrel/fisheye distortion.
/// - **Radial Fold** — float slider (0–1); concentric crystalline rings.
/// - **Möbius** — checkbox; alternate-segment inversion.
/// - **Recursion** — integer step-slider (0–3); recursive reflection passes.
///
/// **Color transforms** (M6):
/// - **Hue** — float slider (0–360°); rotates hue around the colour wheel.
/// - **Saturation** — float slider (0–200%); 100% = unchanged.
/// - **Brightness** — float slider (0–200%); 100% = unchanged.
/// - **Posterize** — integer slider (0=Off, 2–16); quantises to N colour bands.
/// - **Invert** — checkbox; complements all colour channels.
///
/// Each control writes directly to the corresponding [`KaleidoscopeParams`]
/// signal, which triggers a reactive redraw in `CanvasView`.
#[component]
pub fn ControlsPanel() -> impl IntoView {
    let params = expect_context::<KaleidoscopeParams>();
    let app_state = expect_context::<AppState>();

    view! {
        <div class="controls-panel">
            // ================================================================
            // Symmetry controls (M4)
            // ================================================================

            // --- Segments ---------------------------------------------------
            <div class="control-row">
                <label class="control-label">"Segments"</label>
                <input
                    type="range"
                    min="2"
                    max="10"
                    step="1"
                    prop:value=move || params.segments.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: u32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(6);
                        params.segments.set(val.clamp(2, 10));
                    }
                />
                <span class="control-value">
                    {move || params.segments.get().to_string()}
                </span>
            </div>

            // --- Rotation ---------------------------------------------------
            <div class="control-row">
                <label class="control-label">"Rotation"</label>
                <input
                    type="range"
                    min="0"
                    max="360"
                    step="1"
                    prop:value=move || params.rotation.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: f32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(0.0);
                        params.rotation.set(val.clamp(0.0, 360.0));
                    }
                />
                <span class="control-value">
                    {move || format!("{:.0}°", params.rotation.get())}
                </span>
            </div>

            // --- Zoom -------------------------------------------------------
            <div class="control-row">
                <label class="control-label">"Zoom"</label>
                <input
                    type="range"
                    min="0.1"
                    max="4.0"
                    step="0.05"
                    prop:value=move || params.zoom.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: f32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(1.0);
                        params.zoom.set(val.clamp(0.1, 4.0));
                    }
                />
                <span class="control-value">
                    {move || format!("{:.2}×", params.zoom.get())}
                </span>
            </div>

            // --- Center (canvas-drag only; display only here) ---------------
            <div class="control-row">
                <span class="control-label">"Center"</span>
                <span class="control-hint">"drag canvas"</span>
                <span class="control-value">
                    {move || {
                        let (cx, cy) = params.center.get();
                        format!("({:.2}, {:.2})", cx, cy)
                    }}
                </span>
            </div>

            // ================================================================
            // Visual effects (M5)
            // ================================================================

            // --- Spiral -----------------------------------------------------
            {float_slider!(
                label  = "Spiral",
                signal = params.spiral,
                min    = 0.0_f32,
                max    = 1.0_f32,
                step   = 0.01_f32,
                fmt    = "{:.2}"
            )}

            // --- Ripple -----------------------------------------------------
            {float_slider!(
                label  = "Ripple",
                signal = params.ripple,
                min    = 0.0_f32,
                max    = 1.0_f32,
                step   = 0.01_f32,
                fmt    = "{:.2}"
            )}

            // --- Lens -------------------------------------------------------
            {float_slider!(
                label  = "Lens",
                signal = params.lens,
                min    = 0.0_f32,
                max    = 1.0_f32,
                step   = 0.01_f32,
                fmt    = "{:.2}"
            )}

            // --- Radial Fold ------------------------------------------------
            {float_slider!(
                label  = "Radial Fold",
                signal = params.radial_fold,
                min    = 0.0_f32,
                max    = 1.0_f32,
                step   = 0.01_f32,
                fmt    = "{:.2}"
            )}

            // --- Möbius (toggle) --------------------------------------------
            <div class="control-row">
                <label class="control-label">"Möbius"</label>
                <input
                    type="checkbox"
                    prop:checked=move || params.mobius.get()
                    on:change=move |ev: web_sys::Event| {
                        let checked = ev
                            .target()
                            .expect("change event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .checked();
                        params.mobius.set(checked);
                    }
                />
            </div>

            // --- Recursive Depth (step slider 0–3) --------------------------
            <div class="control-row">
                <label class="control-label">"Recursion"</label>
                <input
                    type="range"
                    min="0"
                    max="3"
                    step="1"
                    prop:value=move || params.recursive_depth.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: u32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(0);
                        params.recursive_depth.set(val.clamp(0, 3));
                    }
                />
                <span class="control-value">
                    {move || params.recursive_depth.get().to_string()}
                </span>
            </div>

            // ================================================================
            // Color transforms (M6)
            // ================================================================

            // --- Hue -----------------------------------------------------------
            {float_slider!(
                label  = "Hue",
                signal = params.hue_shift,
                min    = 0.0_f32,
                max    = 360.0_f32,
                step   = 1.0_f32,
                fmt    = "{:.0}°"
            )}

            // --- Saturation -------------------------------------------------
            <div class="control-row">
                <label class="control-label">"Saturation"</label>
                <input
                    type="range"
                    min="0.0"
                    max="2.0"
                    step="0.01"
                    prop:value=move || params.saturation.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: f32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(1.0);
                        params.saturation.set(val.clamp(0.0, 2.0));
                    }
                />
                <span class="control-value">
                    {move || format!("{:.0}%", params.saturation.get() * 100.0)}
                </span>
            </div>

            // --- Brightness -------------------------------------------------
            <div class="control-row">
                <label class="control-label">"Brightness"</label>
                <input
                    type="range"
                    min="0.0"
                    max="2.0"
                    step="0.01"
                    prop:value=move || params.brightness.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: f32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(1.0);
                        params.brightness.set(val.clamp(0.0, 2.0));
                    }
                />
                <span class="control-value">
                    {move || format!("{:.0}%", params.brightness.get() * 100.0)}
                </span>
            </div>

            // --- Posterize (0 = Off, 2–16 = levels) -------------------------
            <div class="control-row">
                <label class="control-label">"Posterize"</label>
                <input
                    type="range"
                    min="0"
                    max="16"
                    step="1"
                    prop:value=move || params.posterize.get().to_string()
                    on:input=move |ev: web_sys::Event| {
                        let val: u32 = ev
                            .target()
                            .expect("input event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .value()
                            .parse()
                            .unwrap_or(0);
                        params.posterize.set(val.clamp(0, 16));
                    }
                />
                <span class="control-value">
                    {move || {
                        let v = params.posterize.get();
                        if v == 0 { "Off".to_string() } else { v.to_string() }
                    }}
                </span>
            </div>

            // --- Invert (toggle) --------------------------------------------
            <div class="control-row">
                <label class="control-label">"Invert"</label>
                <input
                    type="checkbox"
                    prop:checked=move || params.invert.get()
                    on:change=move |ev: web_sys::Event| {
                        let checked = ev
                            .target()
                            .expect("change event has a target")
                            .unchecked_into::<web_sys::HtmlInputElement>()
                            .checked();
                        params.invert.set(checked);
                    }
                />
            </div>

            // ================================================================
            // Randomize (M9)
            // ================================================================
            <button
                class="surprise-button"
                disabled=move || !app_state.image_loaded.get()
                on:click=move |_| randomize(params)
            >
                "⚡ SURPRISE ME"
            </button>

            // ================================================================
            // Export (M8)
            // ================================================================
            <ExportMenu/>
        </div>
    }
}

