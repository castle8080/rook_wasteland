//! Obtain and wrap a WebGL2 rendering context from an `HtmlCanvasElement`.

use wasm_bindgen::JsCast;

/// Obtain a `glow::Context` from an `HtmlCanvasElement`.
///
/// Returns `Err` if the browser does not support WebGL 2 or the context
/// handle cannot be downcast to `WebGl2RenderingContext`.
pub fn get_context(canvas: &web_sys::HtmlCanvasElement) -> Result<glow::Context, String> {
    let raw = canvas
        .get_context("webgl2")
        .map_err(|e| format!("get_context error: {:?}", e))?
        .ok_or_else(|| "WebGL 2 is not supported in this browser".to_string())?;

    let webgl2: web_sys::WebGl2RenderingContext = raw
        .dyn_into()
        .map_err(|_| "context object is not a WebGl2RenderingContext".to_string())?;

    Ok(glow::Context::from_webgl2_context(webgl2))
}

