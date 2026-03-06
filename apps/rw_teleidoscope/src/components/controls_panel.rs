use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::state::KaleidoscopeParams;

/// Side panel containing the four core kaleidoscope controls for M4.
///
/// - **Segments** — integer slider (2–10); controls mirror count.
/// - **Rotation** — float slider (0–360°); rotates the whole pattern.
/// - **Zoom** — float slider (0.1–4.0); scales the source sampling radius.
/// - **Center** — read-only display; updated by dragging directly on the canvas.
///
/// Each slider writes directly to the corresponding [`KaleidoscopeParams`]
/// signal, which triggers a reactive redraw in `CanvasView`.
#[component]
pub fn ControlsPanel() -> impl IntoView {
    let params = expect_context::<KaleidoscopeParams>();

    view! {
        <div class="controls-panel">
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
        </div>
    }
}

