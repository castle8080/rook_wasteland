// Browser integration tests for the ExportMenu component (M8).
//
// Tests verify the disabled state of the EXPORT button, dropdown toggle
// behaviour, and format radio persistence — all observable via the DOM.
// The actual download (Blob → anchor click) is covered by the manual test
// checklist; it cannot be verified headlessly without special browser APIs.
#![cfg(target_arch = "wasm32")]

use leptos::mount::mount_to;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Helpers (mirrors integration.rs)
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
// Tests
// ---------------------------------------------------------------------------

/// EXPORT button is disabled before any image is loaded.
#[wasm_bindgen_test]
async fn export_button_disabled_when_no_image() {
    use leptos::prelude::RwSignal;
    use rw_teleidoscope::state::AppState;

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        leptos::prelude::provide_context(AppState {
            image_loaded: RwSignal::new(false),
            camera_open: RwSignal::new(false),
            camera_error: RwSignal::new(None),
        });
        leptos::prelude::provide_context(rw_teleidoscope::state::KaleidoscopeParams::new());
        leptos::prelude::view! {
            <rw_teleidoscope::components::export_menu::ExportMenu/>
        }
    });
    tick().await;

    let btn = container
        .query_selector(".export-button")
        .unwrap()
        .expect(".export-button should be rendered")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    assert!(btn.disabled(), "EXPORT button should be disabled before an image is loaded");
}

/// EXPORT button becomes enabled after `image_loaded` is set to true.
#[wasm_bindgen_test]
async fn export_button_enabled_after_image_loaded() {
    use leptos::prelude::{RwSignal, Set};
    use rw_teleidoscope::state::AppState;

    // Mount the app with a pre-set image_loaded signal.
    let image_loaded = RwSignal::new(false);

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        leptos::prelude::provide_context(AppState {
            image_loaded,
            camera_open: RwSignal::new(false),
            camera_error: RwSignal::new(None),
        });
        leptos::prelude::provide_context(rw_teleidoscope::state::KaleidoscopeParams::new());
        leptos::prelude::view! {
            <rw_teleidoscope::components::export_menu::ExportMenu/>
        }
    });
    tick().await;

    let btn = container
        .query_selector(".export-button")
        .unwrap()
        .expect(".export-button should be present")
        .dyn_into::<web_sys::HtmlButtonElement>()
        .unwrap();

    assert!(btn.disabled(), "should start disabled");

    image_loaded.set(true);
    tick().await;

    assert!(!btn.disabled(), "EXPORT button should be enabled after image_loaded = true");
}

/// Clicking EXPORT toggles the dropdown into view.
#[wasm_bindgen_test]
async fn export_toggle_opens_dropdown() {
    use leptos::prelude::RwSignal;
    use rw_teleidoscope::state::AppState;

    let image_loaded = RwSignal::new(true); // start with image loaded so button is enabled

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        leptos::prelude::provide_context(AppState {
            image_loaded,
            camera_open: RwSignal::new(false),
            camera_error: RwSignal::new(None),
        });
        leptos::prelude::provide_context(rw_teleidoscope::state::KaleidoscopeParams::new());
        leptos::prelude::view! {
            <rw_teleidoscope::components::export_menu::ExportMenu/>
        }
    });
    tick().await;

    // Dropdown should not be visible initially.
    assert!(
        container.query_selector(".export-dropdown").unwrap().is_none(),
        "dropdown should be hidden initially"
    );

    // Click the EXPORT button.
    let btn = container
        .query_selector(".export-button")
        .unwrap()
        .expect(".export-button should exist")
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    btn.click();
    tick().await;

    // Dropdown should now be visible.
    assert!(
        container.query_selector(".export-dropdown").unwrap().is_some(),
        "dropdown should appear after clicking EXPORT"
    );
}

/// Clicking EXPORT a second time closes the dropdown.
#[wasm_bindgen_test]
async fn export_toggle_closes_dropdown() {
    use leptos::prelude::RwSignal;
    use rw_teleidoscope::state::AppState;

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        leptos::prelude::provide_context(AppState {
            image_loaded: RwSignal::new(true),
            camera_open: RwSignal::new(false),
            camera_error: RwSignal::new(None),
        });
        leptos::prelude::provide_context(rw_teleidoscope::state::KaleidoscopeParams::new());
        leptos::prelude::view! {
            <rw_teleidoscope::components::export_menu::ExportMenu/>
        }
    });
    tick().await;

    let btn = container
        .query_selector(".export-button")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();

    btn.click();
    tick().await;
    assert!(container.query_selector(".export-dropdown").unwrap().is_some(), "open after 1st click");

    btn.click();
    tick().await;
    assert!(container.query_selector(".export-dropdown").unwrap().is_none(), "closed after 2nd click");
}

/// The dropdown contains three format options (PNG, JPEG, WebP).
#[wasm_bindgen_test]
async fn dropdown_has_three_format_options() {
    use leptos::prelude::RwSignal;
    use rw_teleidoscope::state::AppState;

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        leptos::prelude::provide_context(AppState {
            image_loaded: RwSignal::new(true),
            camera_open: RwSignal::new(false),
            camera_error: RwSignal::new(None),
        });
        leptos::prelude::provide_context(rw_teleidoscope::state::KaleidoscopeParams::new());
        leptos::prelude::view! {
            <rw_teleidoscope::components::export_menu::ExportMenu/>
        }
    });
    tick().await;

    let btn = container
        .query_selector(".export-button")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    btn.click();
    tick().await;

    let radios = container
        .query_selector_all("input[type='radio']")
        .unwrap();
    assert_eq!(radios.length(), 3, "should have exactly 3 format radio buttons");
}
