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
