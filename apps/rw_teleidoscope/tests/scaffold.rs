use wasm_bindgen_test::*;
use web_sys::HtmlCanvasElement;
use wasm_bindgen::JsCast;

// Integration tests are a separate compilation unit from the library, so they
// need their own configure! call — the one in src/lib.rs does not cover this file.
wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// Browser environment smoke tests (M1)
// ---------------------------------------------------------------------------

/// `window` must be present — every DOM / WebGL API hangs off it.
#[wasm_bindgen_test]
fn window_is_available() {
    assert!(web_sys::window().is_some(), "window should be available in browser");
}

/// `document` must be present and have a valid `<body>`.
#[wasm_bindgen_test]
fn document_and_body_are_available() {
    let win = web_sys::window().unwrap();
    let doc = win.document().expect("document should be available");
    assert!(doc.body().is_some(), "document.body should exist");
}

/// We must be able to create an `HtmlCanvasElement`.
/// This is the element that will host the WebGL context in later milestones.
#[wasm_bindgen_test]
fn canvas_element_can_be_created() {
    let win = web_sys::window().unwrap();
    let doc = win.document().unwrap();
    let el = doc
        .create_element("canvas")
        .expect("create_element('canvas') should succeed");
    let canvas = el
        .dyn_into::<HtmlCanvasElement>()
        .expect("canvas element should cast to HtmlCanvasElement");
    // Default canvas dimensions as a basic sanity check.
    assert_eq!(canvas.width(), 300);
    assert_eq!(canvas.height(), 150);
}

/// `WebGl2RenderingContext` must be obtainable from a canvas.
/// A pass here confirms WebGL 2 is available in the test browser and that
/// our `glow`-based renderer will be able to acquire a context.
#[wasm_bindgen_test]
fn webgl2_context_is_available() {
    let win = web_sys::window().unwrap();
    let doc = win.document().unwrap();
    let canvas = doc
        .create_element("canvas")
        .unwrap()
        .dyn_into::<HtmlCanvasElement>()
        .unwrap();
    let ctx = canvas
        .get_context("webgl2")
        .expect("get_context('webgl2') should not throw");
    assert!(
        ctx.is_some(),
        "webgl2 context should be available in headless browser"
    );
}

/// `navigator.media_devices()` must be accessible (camera / getUserMedia path).
/// Note: We only check that the property is reachable — actually calling
/// `getUserMedia` in a headless browser would fail with a PermissionDeniedError,
/// which is tested in M7.
#[wasm_bindgen_test]
fn media_devices_is_reachable() {
    let win = web_sys::window().unwrap();
    let nav = win.navigator();
    // media_devices() returns a Result; an Ok(Some(_)) means the property exists.
    let _devices = nav.media_devices().expect("navigator.mediaDevices should exist");
}
