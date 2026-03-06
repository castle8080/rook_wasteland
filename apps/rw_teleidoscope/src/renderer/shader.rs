//! Load, compile, and link GLSL shader source into a WebGL program.

use glow::HasContext;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// Fetch the text content of `url` using the browser `fetch` API.
async fn fetch_text(url: &str) -> Result<String, String> {
    let window = web_sys::window().ok_or("no window")?;

    let resp_val = JsFuture::from(window.fetch_with_str(url))
        .await
        .map_err(|e| format!("fetch {url} failed: {:?}", e))?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "fetch result is not a Response".to_string())?;

    if !resp.ok() {
        return Err(format!("HTTP {} fetching {url}", resp.status()));
    }

    let text_promise = resp
        .text()
        .map_err(|e| format!("resp.text() failed: {:?}", e))?;

    let text_val = JsFuture::from(text_promise)
        .await
        .map_err(|e| format!("text future failed: {:?}", e))?;

    text_val
        .as_string()
        .ok_or_else(|| "response text was not a string".to_string())
}

/// Compile a single shader stage.
///
/// On failure the GLSL info log is returned as `Err` and the shader object
/// is deleted.
unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = gl
        .create_shader(shader_type)
        .map_err(|e| format!("create_shader: {e}"))?;
    gl.shader_source(shader, source);
    gl.compile_shader(shader);

    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        let log = gl.get_shader_info_log(shader);
        gl.delete_shader(shader);
        Err(format!("Shader compile error: {log}"))
    }
}

/// Fetch vertex and fragment GLSL sources from `assets/shaders/`, compile
/// them, and link them into a `glow::Program`.
///
/// Compilation and link errors are written to `console.error` before the
/// `Err` is returned.
pub async fn create_program(gl: &glow::Context) -> Result<glow::Program, String> {
    let vert_src =
        fetch_text("/rw_teleidoscope/shaders/vert.glsl").await?;
    let frag_src =
        fetch_text("/rw_teleidoscope/shaders/frag.glsl").await?;

    unsafe {
        let vert = compile_shader(gl, glow::VERTEX_SHADER, &vert_src).inspect_err(|e| {
            web_sys::console::error_1(&e.clone().into());
        })?;
        let frag =
            compile_shader(gl, glow::FRAGMENT_SHADER, &frag_src).inspect_err(|e| {
                web_sys::console::error_1(&e.clone().into());
            })?;

        let program = gl
            .create_program()
            .map_err(|e| format!("create_program: {e}"))?;
        gl.attach_shader(program, vert);
        gl.attach_shader(program, frag);
        gl.link_program(program);

        // Shaders are no longer needed once linked.
        gl.detach_shader(program, vert);
        gl.delete_shader(vert);
        gl.detach_shader(program, frag);
        gl.delete_shader(frag);

        if gl.get_program_link_status(program) {
            Ok(program)
        } else {
            let log = gl.get_program_info_log(program);
            gl.delete_program(program);
            let msg = format!("Program link error: {log}");
            web_sys::console::error_1(&msg.clone().into());
            Err(msg)
        }
    }
}

