// Integration tests for Feature 001: Mobile / Responsive Layout (M11).
//
// These tests verify the signal wiring for the mobile bottom drawer.  CSS
// media-query effects are NOT testable here (the headless Firefox viewport is
// larger than 768 px so the breakpoint never fires). Visual layout must be
// verified manually via the checklist in features/feature_001_mobile_responsive_layout.md.
//
// What IS verified:
//   • `.drawer-handle` and `.drawer-backdrop` are always in the DOM
//   • Clicking `.drawer-handle` gives `.controls-panel` the `drawer--open` class
//   • `.drawer-backdrop` gains `is-visible` when the drawer is open
//   • Clicking `.drawer-backdrop` removes both classes
#![cfg(target_arch = "wasm32")]

use leptos::mount::mount_to;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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
// Test: the full App mounts with the drawer handle and backdrop in the DOM
// ---------------------------------------------------------------------------

/// The full `App` must render `.drawer-handle` and `.drawer-backdrop` elements
/// unconditionally (always in DOM; CSS hides them on desktop).
/// On load the backdrop must NOT carry the `is-visible` class.
#[wasm_bindgen_test]
async fn app_mounts_with_drawer_signal() {
    let container = fresh_container();
    let _handle = mount_to(container.clone(), rw_teleidoscope::app::App);
    tick().await;

    assert!(
        container.query_selector(".drawer-handle").unwrap().is_some(),
        ".drawer-handle must be in the DOM"
    );
    assert!(
        container.query_selector(".drawer-backdrop").unwrap().is_some(),
        ".drawer-backdrop must be in the DOM"
    );

    let backdrop = container.query_selector(".drawer-backdrop").unwrap().unwrap();
    assert!(
        !backdrop.class_list().contains("is-visible"),
        ".drawer-backdrop must not have is-visible on load"
    );
}

// ---------------------------------------------------------------------------
// Test: clicking the drawer handle toggles drawer--open on the controls panel
// ---------------------------------------------------------------------------

/// Clicking `.drawer-handle` must set `drawer_open` to `true`, causing
/// `.controls-panel` to gain the `drawer--open` CSS class and `.drawer-backdrop`
/// to gain `is-visible`.
#[wasm_bindgen_test]
async fn drawer_toggles_on_handle_click() {
    let container = fresh_container();
    let _handle = mount_to(container.clone(), rw_teleidoscope::app::App);
    tick().await;

    let panel = container
        .query_selector(".controls-panel")
        .unwrap()
        .expect(".controls-panel must exist");

    assert!(
        !panel.class_list().contains("drawer--open"),
        "drawer--open must not be set before handle click"
    );

    let handle_btn: web_sys::HtmlElement = container
        .query_selector(".drawer-handle")
        .unwrap()
        .expect(".drawer-handle must exist")
        .unchecked_into();
    handle_btn.click();
    tick().await;

    assert!(
        panel.class_list().contains("drawer--open"),
        ".controls-panel must have drawer--open after handle click"
    );

    let backdrop = container.query_selector(".drawer-backdrop").unwrap().unwrap();
    assert!(
        backdrop.class_list().contains("is-visible"),
        ".drawer-backdrop must have is-visible when drawer is open"
    );
}

// ---------------------------------------------------------------------------
// Test: clicking the backdrop closes the drawer
// ---------------------------------------------------------------------------

/// After opening the drawer, clicking `.drawer-backdrop` must clear
/// `drawer_open`, removing `drawer--open` from the panel and `is-visible`
/// from the backdrop.
#[wasm_bindgen_test]
async fn backdrop_click_closes_drawer() {
    let container = fresh_container();
    let _handle = mount_to(container.clone(), rw_teleidoscope::app::App);
    tick().await;

    // Open the drawer first.
    let handle_btn: web_sys::HtmlElement = container
        .query_selector(".drawer-handle")
        .unwrap()
        .expect(".drawer-handle must exist")
        .unchecked_into();
    handle_btn.click();
    tick().await;

    let panel = container
        .query_selector(".controls-panel")
        .unwrap()
        .expect(".controls-panel must exist");
    assert!(
        panel.class_list().contains("drawer--open"),
        "drawer must be open before backdrop click"
    );

    let backdrop: web_sys::HtmlElement = container
        .query_selector(".drawer-backdrop")
        .unwrap()
        .expect(".drawer-backdrop must exist")
        .unchecked_into();
    backdrop.click();
    tick().await;

    assert!(
        !panel.class_list().contains("drawer--open"),
        ".controls-panel must lose drawer--open after backdrop click"
    );
    assert!(
        !backdrop.class_list().contains("is-visible"),
        ".drawer-backdrop must lose is-visible after close"
    );
}
