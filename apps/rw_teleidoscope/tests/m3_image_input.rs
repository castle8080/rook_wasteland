// This file uses WebGL 2, glow, and web_sys APIs that only exist on wasm32.
// Gating the entire file here means `cargo test` (native) skips it cleanly,
// while `wasm-pack test --headless --firefox` compiles and runs all tests.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

// Integration tests are a separate compilation unit from the library, so they
// need their own configure! call — the one in src/lib.rs does not cover this file.
wasm_bindgen_test_configure!(run_in_browser);

// ---------------------------------------------------------------------------
// M3 — Image Input & Texture Display
// ---------------------------------------------------------------------------

/// `HtmlImageElement` can be constructed in the browser.
///
/// This is a prerequisite for the `FileReader` → image decode path: the
/// pipeline creates one of these elements and sets its `src` to a data URL.
#[wasm_bindgen_test]
fn html_image_element_can_be_created() {
    let img = web_sys::HtmlImageElement::new()
        .expect("HtmlImageElement::new should succeed in a browser context");
    // Before `src` is set the natural dimensions are zero.
    assert_eq!(img.natural_width(), 0, "unloaded image should have width 0");
    assert_eq!(img.natural_height(), 0, "unloaded image should have height 0");
}

/// An offscreen `<canvas>` (not attached to the DOM) can produce `ImageData`.
///
/// This tests the browser infrastructure used by `utils::resize_to_800`:
/// create an element, get a 2D context, and read back pixel data.
#[wasm_bindgen_test]
fn offscreen_canvas_produces_image_data() {
    let doc = web_sys::window()
        .expect("window")
        .document()
        .expect("document");

    let canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")
        .expect("create canvas")
        .dyn_into()
        .expect("HtmlCanvasElement");

    canvas.set_width(8);
    canvas.set_height(8);

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .expect("get_context ok")
        .expect("2d context present")
        .dyn_into()
        .expect("CanvasRenderingContext2d");

    let image_data = ctx
        .get_image_data(0.0, 0.0, 8.0, 8.0)
        .expect("getImageData should succeed");

    assert_eq!(image_data.width(), 8);
    assert_eq!(image_data.height(), 8);
    // Each pixel is 4 bytes (RGBA) → 8×8×4 = 256 bytes total.
    assert_eq!(image_data.data().length(), 256);
}

/// `utils::resize_to_800` returns an `ImageData` of exactly 800×800 pixels.
///
/// A 100×50 offscreen canvas (non-square) is used as the source image.  After
/// `resize_to_800` processes it, the output must be 800×800 with 4 bytes per
/// pixel (RGBA).  This test uses an async callback chain to wait for the
/// `HtmlImageElement` `onload` event before calling the resize function.
#[wasm_bindgen_test]
async fn resize_to_800_returns_correct_dimensions() {
    use wasm_bindgen::prelude::Closure;
    use wasm_bindgen_futures::JsFuture;

    let doc = web_sys::window()
        .expect("window")
        .document()
        .expect("document");

    // Build a 100×50 source canvas and export it as a PNG data URL.
    let src_canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")
        .expect("create canvas")
        .dyn_into()
        .expect("HtmlCanvasElement");
    src_canvas.set_width(100);
    src_canvas.set_height(50);
    let data_url = src_canvas.to_data_url().expect("to_data_url");

    // Load the data URL into an HtmlImageElement and wait for `onload`.
    let img = web_sys::HtmlImageElement::new().expect("HtmlImageElement::new");

    // `Promise::new` runs its executor synchronously, so we can borrow `img`
    // inside the closure and still use `img` after `Promise::new` returns.
    let img_clone = img.clone();
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let cb = Closure::<dyn FnMut()>::new(move || {
            resolve.call0(&wasm_bindgen::JsValue::NULL).unwrap();
        });
        img_clone.set_onload(Some(cb.as_ref().unchecked_ref()));
        cb.forget();
    });

    img.set_src(&data_url);
    JsFuture::from(promise).await.expect("image onload timed out");

    // img now has naturalWidth=100, naturalHeight=50.
    let image_data = rw_teleidoscope::utils::resize_to_800(&img)
        .expect("resize_to_800 should succeed");

    assert_eq!(image_data.width(), 800, "output width must be 800");
    assert_eq!(image_data.height(), 800, "output height must be 800");
    assert_eq!(
        image_data.data().length(),
        800 * 800 * 4,
        "RGBA byte count must be 800*800*4"
    );
}

/// without errors.
///
/// This exercises `renderer::texture::upload_image_data` end-to-end using a
/// 4×4 transparent canvas rather than a real image, keeping the test fast
/// and self-contained.
#[wasm_bindgen_test]
fn texture_upload_succeeds_with_blank_image_data() {
    let doc = web_sys::window()
        .expect("window")
        .document()
        .expect("document");

    // Create source ImageData (4×4 transparent pixels).
    let src_canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")
        .expect("create canvas")
        .dyn_into()
        .expect("HtmlCanvasElement");
    src_canvas.set_width(4);
    src_canvas.set_height(4);
    let ctx: web_sys::CanvasRenderingContext2d = src_canvas
        .get_context("2d")
        .expect("ok")
        .expect("ctx")
        .dyn_into()
        .expect("2d");
    let image_data = ctx
        .get_image_data(0.0, 0.0, 4.0, 4.0)
        .expect("getImageData");

    // Obtain a WebGL 2 context for the upload.
    let gl_canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")
        .expect("create canvas")
        .dyn_into()
        .expect("HtmlCanvasElement");
    let raw_ctx = gl_canvas
        .get_context("webgl2")
        .expect("get_context ok")
        .expect("webgl2 context present")
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .expect("WebGl2RenderingContext");
    let gl = glow::Context::from_webgl2_context(raw_ctx);

    let result = rw_teleidoscope::renderer::texture::upload_image_data(&gl, &image_data);
    assert!(
        result.is_ok(),
        "upload_image_data should succeed: {:?}",
        result.err()
    );
}
