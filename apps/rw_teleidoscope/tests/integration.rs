// Integration tests for the full app lifecycle.
//
// These tests mount real Leptos components into a headless Firefox browser and
// verify the observable DOM/signal behaviour.  They complement the unit tests
// in `m3_image_input.rs` (which test individual pipeline functions) by
// confirming that the components are wired together correctly end-to-end.
//
// Each test mounts into a fresh `<div>` appended to `document.body`, keeping
// tests isolated from each other even though they share the same browser page.
// The `UnmountHandle` returned by `mount_to` is held for the lifetime of the
// test; dropping it cleans up the reactive graph and removes DOM nodes.
//
// **Why shaders can be tested here:** shaders are embedded with `include_str!`
// rather than fetched at runtime, so renderer initialisation succeeds in the
// wasm-pack test environment (which does not serve static asset files).
#![cfg(target_arch = "wasm32")]

use leptos::mount::mount_to;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn doc() -> web_sys::Document {
    web_sys::window().unwrap().document().unwrap()
}

/// Create a fresh `<div>`, append it to `document.body`, and return it.
///
/// Each test gets its own container so DOM queries scoped to that container
/// don't see elements mounted by other tests.
fn fresh_container() -> web_sys::HtmlElement {
    let div: web_sys::HtmlElement = doc()
        .create_element("div")
        .unwrap()
        .unchecked_into();
    doc().body().unwrap().append_child(&div).unwrap();
    div
}

/// Yield once to the microtask queue so Leptos effects can flush.
///
/// `Promise.resolve().then(...)` resolves in the microtask checkpoint, which
/// is where Leptos schedules its reactive effects in 0.8 CSR mode.
async fn tick() {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(
        &wasm_bindgen::JsValue::NULL,
    ))
    .await
    .unwrap();
}

// ---------------------------------------------------------------------------
// Smoke test: DOM structure
// ---------------------------------------------------------------------------

/// The full app mounts without panicking and renders the expected DOM tree.
///
/// This test exercises the complete startup path:
/// - `App` provides `AppState` and `KaleidoscopeParams` via context
/// - `Header` renders the title and both action buttons
/// - `CanvasView` creates the `<canvas>`, initialises the WebGL renderer
///   synchronously (shaders embedded), and shows the placeholder overlay
///   because no image has been loaded yet
#[wasm_bindgen_test]
async fn app_mounts_with_correct_dom_structure() {
    let container = fresh_container();
    let _handle = mount_to(container.clone(), rw_teleidoscope::app::App);
    tick().await;

    // Canvas must exist.
    assert!(
        container.query_selector("#kaleidoscope-canvas").unwrap().is_some(),
        "canvas#kaleidoscope-canvas must be in the DOM"
    );

    // Before any image is loaded the placeholder overlay must be visible.
    assert!(
        container.query_selector(".canvas-placeholder").unwrap().is_some(),
        "placeholder overlay must be shown before an image is loaded"
    );

    // Header must be present with at least the two action buttons.
    assert!(
        container.query_selector("#app-header").unwrap().is_some(),
        "#app-header must be in the DOM"
    );
    // query_selector_all lives on Element; HtmlElement coerces via JsCast.
    let el = container.unchecked_ref::<web_sys::Element>();
    let buttons = el.query_selector_all(".header-btn").unwrap();
    assert!(buttons.length() >= 2, "at least 2 .header-btn elements expected");
}

// ---------------------------------------------------------------------------
// Reactive overlay: placeholder responds to the image_loaded signal
// ---------------------------------------------------------------------------

/// Setting `AppState.image_loaded` to `true` removes the placeholder overlay.
///
/// This test mounts only `CanvasView` (not the full `App`) with a manually
/// constructed `AppState` so we can control the `image_loaded` signal directly
/// from outside the component.  The test verifies:
/// 1. Overlay is present when `image_loaded = false`
/// 2. Overlay disappears after `image_loaded.set(true)` and one reactive tick
#[wasm_bindgen_test]
async fn placeholder_hides_when_image_loaded_signal_is_set() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        components::canvas_view::CanvasView,
        state::{AppState, KaleidoscopeParams},
    };

    // Signals are created before mount_to; they remain valid for the test
    // duration because RwSignal uses Arc-backed storage in Leptos 0.8.
    let image_loaded: RwSignal<bool> = RwSignal::new(false);
    let state = AppState {
        image_loaded,
        camera_open: RwSignal::new(false),
        camera_error: RwSignal::new(None),
        panel_open: RwSignal::new(true),
        drawer_open: RwSignal::new(false),
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(KaleidoscopeParams::new());
        provide_context(state);
        view! { <CanvasView/> }
    });
    tick().await;

    // Before: overlay must be present.
    assert!(
        container
            .query_selector(".canvas-placeholder")
            .unwrap()
            .is_some(),
        "placeholder must be visible when image_loaded = false"
    );

    // Trigger the reactive update.
    image_loaded.set(true);
    tick().await;

    // After: overlay must be gone.
    assert!(
        container
            .query_selector(".canvas-placeholder")
            .unwrap()
            .is_none(),
        "placeholder must be hidden after image_loaded = true"
    );
}

// ---------------------------------------------------------------------------
// Full pipeline: canvas → resize → upload → signal
// ---------------------------------------------------------------------------

/// The complete M3 image pipeline works end-to-end inside a mounted app.
///
/// Steps:
/// 1. Mount the full `App` and wait for the renderer to initialise.
/// 2. Create a 100 × 50 offscreen canvas as a synthetic image source.
/// 3. Export it as a PNG data URL and load into an `HtmlImageElement`.
/// 4. Call `utils::resize_to_800` → `renderer::upload_image` directly, then
///    retrieve `AppState` via Leptos context and set `image_loaded = true`.
/// 5. Assert the placeholder overlay disappears.
///
/// This is the same data path that `load_file` in `header.rs` exercises at
/// runtime, minus the `FileReader` wrapper (which requires a real `File`
/// object that is difficult to construct synthetically in a browser test).
#[wasm_bindgen_test]
async fn image_pipeline_hides_overlay_end_to_end() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        renderer,
        state::{AppState, KaleidoscopeParams},
        utils,
    };
    use wasm_bindgen::closure::Closure;

    // 1. Mount full app with a known AppState we can control.
    let image_loaded: RwSignal<bool> = RwSignal::new(false);
    let state = AppState {
        image_loaded,
        camera_open: RwSignal::new(false),
        camera_error: RwSignal::new(None),
        panel_open: RwSignal::new(true),
        drawer_open: RwSignal::new(false),
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(KaleidoscopeParams::new());
        provide_context(state);
        view! {
            <rw_teleidoscope::components::canvas_view::CanvasView/>
        }
    });
    tick().await;

    // Renderer should now be initialised (sync init, no fetch).
    assert!(renderer::is_initialized(), "renderer must be ready after tick");

    // 2–3. Build a 100×50 source canvas → data URL → HtmlImageElement.
    let src_canvas: web_sys::HtmlCanvasElement = doc()
        .create_element("canvas")
        .unwrap()
        .unchecked_into();
    src_canvas.set_width(100);
    src_canvas.set_height(50);
    let data_url = src_canvas.to_data_url().unwrap();

    let img = web_sys::HtmlImageElement::new().unwrap();
    let img_clone = img.clone();
    let onload_promise = js_sys::Promise::new(&mut |resolve, _| {
        let cb = Closure::<dyn FnMut()>::new(move || {
            resolve.call0(&wasm_bindgen::JsValue::NULL).unwrap();
        });
        img_clone.set_onload(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
    });
    img.set_src(&data_url);
    wasm_bindgen_futures::JsFuture::from(onload_promise)
        .await
        .unwrap();

    // 4. Run the resize → upload → signal steps (mirrors load_file internals).
    let image_data = utils::resize_to_800(&img).expect("resize_to_800 must succeed");
    renderer::with_renderer_mut(|r| r.upload_image(&image_data));
    image_loaded.set(true);
    tick().await;

    // 5. Placeholder must be gone.
    assert!(
        container
            .query_selector(".canvas-placeholder")
            .unwrap()
            .is_none(),
        "placeholder must be hidden after full pipeline completes"
    );
}

// ---------------------------------------------------------------------------
// M7 Camera overlay: signal → DOM visibility
// ---------------------------------------------------------------------------

/// The camera overlay is absent when `camera_open = false`.
///
/// Mounts `App` and verifies no `.camera-overlay` element exists in the DOM
/// before the user opens the camera.
#[wasm_bindgen_test]
async fn camera_overlay_hidden_when_camera_closed() {
    let container = fresh_container();
    let _handle = mount_to(container.clone(), rw_teleidoscope::app::App);
    tick().await;

    assert!(
        container
            .query_selector(".camera-overlay")
            .unwrap()
            .is_none(),
        "camera-overlay must be absent when camera_open = false"
    );
}

/// Setting `camera_open = true` causes the overlay to appear in the DOM.
///
/// Mounts a `CameraOverlay` (not the full App) with a manually constructed
/// `AppState` so we can control `camera_open` without triggering a real
/// `getUserMedia` call.  The test verifies:
/// 1. Overlay is absent when `camera_open = false`
/// 2. Overlay appears after `camera_open.set(true)` and one reactive tick
///
/// Note: the Effect inside `CameraOverlay` will call `camera::request_camera()`
/// in the background.  That call may fail in the headless test environment
/// (no real camera), but we only check DOM structure here — not the video feed.
#[wasm_bindgen_test]
async fn camera_overlay_shows_when_camera_open_signal_set() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        components::camera_overlay::CameraOverlay,
        state::AppState,
    };

    let camera_open: RwSignal<bool> = RwSignal::new(false);
    let state = AppState {
        image_loaded: RwSignal::new(false),
        camera_open,
        camera_error: RwSignal::new(None),
        panel_open: RwSignal::new(true),
        drawer_open: RwSignal::new(false),
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(state);
        view! { <CameraOverlay/> }
    });
    tick().await;

    // Before: overlay must be absent.
    assert!(
        container
            .query_selector(".camera-overlay")
            .unwrap()
            .is_none(),
        "overlay must be absent when camera_open = false"
    );

    // Open the overlay.
    camera_open.set(true);
    tick().await;

    // After: overlay must be present.
    assert!(
        container
            .query_selector(".camera-overlay")
            .unwrap()
            .is_some(),
        "overlay must appear when camera_open = true"
    );
}

/// Setting `camera_error` shows the inline error message inside the overlay.
#[wasm_bindgen_test]
async fn camera_overlay_shows_error_message() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        components::camera_overlay::CameraOverlay,
        state::AppState,
    };

    let camera_open: RwSignal<bool> = RwSignal::new(true);
    let camera_error: RwSignal<Option<String>> = RwSignal::new(None);
    let state = AppState {
        image_loaded: RwSignal::new(false),
        camera_open,
        camera_error,
        panel_open: RwSignal::new(true),
        drawer_open: RwSignal::new(false),
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(state);
        view! { <CameraOverlay/> }
    });
    tick().await;

    // Before: no error element.
    assert!(
        container.query_selector(".camera-error").unwrap().is_none(),
        "no .camera-error should exist before an error is set"
    );

    // Inject an error.
    camera_error.set(Some("Permission denied".to_string()));
    tick().await;

    // After: error message must be visible.
    let error_el = container
        .query_selector(".camera-error")
        .unwrap()
        .expect(".camera-error must appear after camera_error signal is set");

    assert_eq!(
        error_el.text_content().unwrap_or_default(),
        "Permission denied",
        "error text must match the signal value"
    );
}

// ---------------------------------------------------------------------------
// M9 Randomize: "Surprise Me" button presence and disabled state
// ---------------------------------------------------------------------------

/// The "Surprise Me" button is present in the DOM when the app is mounted.
///
/// Verifies that `ControlsPanel` renders the `.surprise-button` element.
#[wasm_bindgen_test]
async fn surprise_me_button_present_in_dom() {
    let container = fresh_container();
    let _handle = mount_to(container.clone(), rw_teleidoscope::app::App);
    tick().await;

    assert!(
        container
            .query_selector(".surprise-button")
            .unwrap()
            .is_some(),
        ".surprise-button must be in the DOM after app mounts"
    );
}

/// The "Surprise Me" button is disabled when no image is loaded.
///
/// Mounts `ControlsPanel` with `image_loaded = false` and checks that the
/// button carries the `disabled` attribute, matching the same pattern used
/// by the EXPORT button.
#[wasm_bindgen_test]
async fn surprise_me_button_disabled_when_no_image() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        components::controls_panel::ControlsPanel,
        state::{AppState, KaleidoscopeParams},
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(KaleidoscopeParams::new());
        provide_context(AppState::new());
        view! { <ControlsPanel/> }
    });
    tick().await;

    let btn = container
        .query_selector(".surprise-button")
        .unwrap()
        .expect(".surprise-button must be rendered");

    assert!(
        btn.has_attribute("disabled"),
        ".surprise-button must be disabled when image_loaded = false"
    );
}

/// The "Surprise Me" button becomes enabled once an image is loaded.
#[wasm_bindgen_test]
async fn surprise_me_button_enabled_when_image_loaded() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        components::controls_panel::ControlsPanel,
        state::{AppState, KaleidoscopeParams},
    };

    let image_loaded: RwSignal<bool> = RwSignal::new(false);
    let state = AppState {
        image_loaded,
        camera_open: RwSignal::new(false),
        camera_error: RwSignal::new(None),
        panel_open: RwSignal::new(true),
        drawer_open: RwSignal::new(false),
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(KaleidoscopeParams::new());
        provide_context(state);
        view! { <ControlsPanel/> }
    });
    tick().await;

    // Before: button disabled.
    let btn = container
        .query_selector(".surprise-button")
        .unwrap()
        .expect(".surprise-button must be rendered");
    assert!(
        btn.has_attribute("disabled"),
        "button must be disabled before image loads"
    );

    // Load image.
    image_loaded.set(true);
    tick().await;

    let btn = container
        .query_selector(".surprise-button")
        .unwrap()
        .expect(".surprise-button must still be in the DOM");
    assert!(
        !btn.has_attribute("disabled"),
        "button must be enabled after image_loaded = true"
    );
}

// ---------------------------------------------------------------------------
// M10: Collapsible panel
// ---------------------------------------------------------------------------

/// Setting `AppState.panel_open` to `false` adds the `is-collapsed` class to
/// the controls panel; setting it back to `true` removes it.
#[wasm_bindgen_test]
async fn panel_collapses_when_panel_open_is_false() {
    use leptos::prelude::*;
    use rw_teleidoscope::{
        components::controls_panel::ControlsPanel,
        state::{AppState, KaleidoscopeParams},
    };

    let panel_open: RwSignal<bool> = RwSignal::new(true);
    let state = AppState {
        image_loaded: RwSignal::new(false),
        camera_open:  RwSignal::new(false),
        camera_error: RwSignal::new(None),
        panel_open,
        drawer_open: RwSignal::new(false),
    };

    let container = fresh_container();
    let _handle = mount_to(container.clone(), move || {
        provide_context(KaleidoscopeParams::new());
        provide_context(state);
        view! { <ControlsPanel/> }
    });
    tick().await;

    // Panel starts open — no is-collapsed class.
    let panel_el = container
        .query_selector(".controls-panel")
        .unwrap()
        .expect(".controls-panel must be in the DOM");
    assert!(
        !panel_el.class_list().contains("is-collapsed"),
        "panel must not have is-collapsed class when panel_open = true"
    );

    // Collapse the panel.
    panel_open.set(false);
    tick().await;

    let panel_el = container
        .query_selector(".controls-panel")
        .unwrap()
        .expect(".controls-panel must still be in the DOM");
    assert!(
        panel_el.class_list().contains("is-collapsed"),
        "panel must have is-collapsed class when panel_open = false"
    );

    // Re-expand.
    panel_open.set(true);
    tick().await;

    let panel_el = container
        .query_selector(".controls-panel")
        .unwrap()
        .expect(".controls-panel must still be in the DOM");
    assert!(
        !panel_el.class_list().contains("is-collapsed"),
        "panel must lose is-collapsed class when panel_open is set back to true"
    );
}
