//! Obtain and wrap a WebGL2 rendering context from an `HtmlCanvasElement`.

use wasm_bindgen::JsCast;

/// Obtain a `glow::Context` from an `HtmlCanvasElement`.
///
/// Returns `Err` if the browser does not support WebGL 2 or the context
/// handle cannot be downcast to `WebGl2RenderingContext`.
///
/// The context is created with `preserveDrawingBuffer: true` so that
/// `canvas.toBlob()` — which runs asynchronously after the current JS task —
/// reads the actual rendered pixels instead of the cleared buffer that the
/// browser would otherwise substitute after compositing.
pub fn get_context(canvas: &web_sys::HtmlCanvasElement) -> Result<glow::Context, String> {
    // Build { preserveDrawingBuffer: true }.
    // Without this flag the browser may clear the drawing buffer immediately
    // after compositing each frame.  canvas.toBlob() is async and fires on the
    // next event-loop tick, so it would always capture the cleared buffer —
    // producing transparent PNGs and solid-black JPEGs.
    let opts = js_sys::Object::new();
    js_sys::Reflect::set(
        &opts,
        &wasm_bindgen::JsValue::from_str("preserveDrawingBuffer"),
        &wasm_bindgen::JsValue::TRUE,
    )
    .map_err(|e| format!("Reflect::set error: {e:?}"))?;

    let raw = canvas
        .get_context_with_context_options("webgl2", &opts)
        .map_err(|e| format!("get_context error: {e:?}"))?
        .ok_or_else(|| "WebGL 2 is not supported in this browser".to_string())?;

    let webgl2: web_sys::WebGl2RenderingContext = raw
        .dyn_into()
        .map_err(|_| "context object is not a WebGl2RenderingContext".to_string())?;

    Ok(glow::Context::from_webgl2_context(webgl2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Verify that `get_context` creates a WebGL2 context with
    /// `preserveDrawingBuffer: true`.
    ///
    /// `canvas.toBlob()` is asynchronous — it fires after the current JS task
    /// completes.  Without `preserveDrawingBuffer: true` the browser may clear
    /// the drawing buffer after compositing, so `toBlob` would capture empty
    /// pixels (transparent PNG / solid-black JPEG).  This test guards against
    /// that regression.
    #[wasm_bindgen_test]
    fn context_has_preserve_drawing_buffer() {
        let doc = web_sys::window()
            .expect("window")
            .document()
            .expect("document");

        let canvas: web_sys::HtmlCanvasElement = doc
            .create_element("canvas")
            .expect("create canvas")
            .unchecked_into();
        canvas.set_width(8);
        canvas.set_height(8);

        // Call our wrapper (which sets preserveDrawingBuffer: true).
        get_context(&canvas).expect("WebGL2 should be available in headless Firefox");

        // Re-fetch the same context from the canvas (browsers reuse the
        // existing context for subsequent get_context calls on the same canvas).
        let raw = canvas
            .get_context("webgl2")
            .expect("get_context ok")
            .expect("context exists after get_context()");

        let webgl2: web_sys::WebGl2RenderingContext = raw.unchecked_into();
        let attrs = webgl2
            .get_context_attributes()
            .expect("context attributes should be available");

        // web-sys 0.3.91 WebGlContextAttributes uses builder-style setters;
        // read the underlying JS property via Reflect to check the value.
        let pdb = js_sys::Reflect::get(
            attrs.as_ref(),
            &wasm_bindgen::JsValue::from_str("preserveDrawingBuffer"),
        )
        .expect("Reflect::get preserveDrawingBuffer");

        assert_eq!(
            pdb.as_bool(),
            Some(true),
            "WebGL2 context must have preserveDrawingBuffer: true \
             so that canvas.toBlob() captures rendered pixels rather than a cleared buffer"
        );
    }
}

