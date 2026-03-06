//! Compile and link embedded GLSL shader sources into a WebGL program.
//!
//! Shader sources are embedded at compile time with `include_str!()`.  This
//! eliminates the runtime network round-trip that was previously needed to
//! fetch the `.glsl` files, making renderer initialisation synchronous and
//! allowing the renderer to be constructed in browser integration tests
//! without a static-asset server.

use glow::HasContext;

/// Vertex shader GLSL source, embedded at compile time.
const VERT_SRC: &str = include_str!("../../assets/shaders/vert.glsl");

/// Fragment shader GLSL source, embedded at compile time.
const FRAG_SRC: &str = include_str!("../../assets/shaders/frag.glsl");

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

/// Compile the embedded GLSL sources and link them into a `glow::Program`.
///
/// Compilation and link errors are written to `console.error` before the
/// `Err` is returned.
pub fn create_program(gl: &glow::Context) -> Result<glow::Program, String> {
    unsafe {
        let vert = compile_shader(gl, glow::VERTEX_SHADER, VERT_SRC).inspect_err(|e| {
            web_sys::console::error_1(&e.clone().into());
        })?;
        let frag = compile_shader(gl, glow::FRAGMENT_SHADER, FRAG_SRC).inspect_err(|e| {
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

