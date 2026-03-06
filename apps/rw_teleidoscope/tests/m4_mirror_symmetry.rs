// Browser integration tests for M4 — Mirror Symmetry Core.
//
// Tests mount real Leptos components into headless Firefox and verify that:
// - The controls panel renders the expected sliders and value spans.
// - Signal mutations are reflected in the control-value display.
#![cfg(target_arch = "wasm32")]

use leptos::mount::mount_to;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn doc() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

fn fresh_container() -> web_sys::HtmlElement {
    let div: web_sys::HtmlElement = doc()
        .create_element("div")
        .unwrap()
        .unchecked_into();
    doc().body().unwrap().append_child(&div).unwrap();
    div
}

async fn tick() {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(
        &wasm_bindgen::JsValue::NULL,
    ))
    .await
    .unwrap();
}

// ---------------------------------------------------------------------------
// Controls panel renders expected sliders
// ---------------------------------------------------------------------------

/// Mounting `ControlsPanel` with a `KaleidoscopeParams` context produces
/// the three range sliders (segments, rotation, zoom) and a center display.
#[wasm_bindgen_test]
async fn controls_panel_renders_sliders() {
    use rw_teleidoscope::{
        components::controls_panel::ControlsPanel,
        state::KaleidoscopeParams,
    };

    let params = KaleidoscopeParams::new();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(params);
        view! { <ControlsPanel/> }
    });
    tick().await;

    let el = container.unchecked_ref::<web_sys::Element>();
    let sliders = el.query_selector_all("input[type='range']").unwrap();
    assert_eq!(sliders.length(), 3, "expected 3 range sliders (segments, rotation, zoom)");

    let value_spans = el.query_selector_all(".control-value").unwrap();
    assert!(
        value_spans.length() >= 3,
        "expected at least 3 .control-value spans"
    );
}

// ---------------------------------------------------------------------------
// Signal changes are reflected in the value display
// ---------------------------------------------------------------------------

/// Mutating `params.segments` outside the component updates the displayed
/// value span after one reactive tick.
#[wasm_bindgen_test]
async fn controls_panel_segments_value_updates_reactively() {
    use rw_teleidoscope::{
        components::controls_panel::ControlsPanel,
        state::KaleidoscopeParams,
    };

    let params = KaleidoscopeParams::new();
    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(params);
        view! { <ControlsPanel/> }
    });
    tick().await;

    // Default is 6 — first value span should show "6".
    let el = container.unchecked_ref::<web_sys::Element>();
    let first_span = el
        .query_selector(".control-value")
        .unwrap()
        .expect("at least one .control-value span");
    assert_eq!(
        first_span.text_content().unwrap_or_default(),
        "6",
        "default segments display should be '6'"
    );

    // Mutate the signal.
    params.segments.set(3);
    tick().await;

    let first_span = el
        .query_selector(".control-value")
        .unwrap()
        .expect(".control-value span");
    assert_eq!(
        first_span.text_content().unwrap_or_default(),
        "3",
        "segments display should update to '3' after signal mutation"
    );
}
