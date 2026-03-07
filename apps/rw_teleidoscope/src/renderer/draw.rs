//! Allocate quad geometry and issue the draw call for one kaleidoscope frame.

use glow::HasContext;

use crate::state::ParamsSnapshot;

use super::uniforms::UniformLocations;

/// Clip-space positions for a full-screen quad (two triangles, 6 vertices).
/// The vertex shader's `a_position` is fixed at `layout(location = 0)`.
#[rustfmt::skip]
const QUAD_VERTS: [f32; 12] = [
    -1.0, -1.0,
     1.0, -1.0,
     1.0,  1.0,
    -1.0, -1.0,
     1.0,  1.0,
    -1.0,  1.0,
];

/// Allocate a VAO and VBO for the full-screen quad.
///
/// Vertex data is uploaded once with `STATIC_DRAW` and never changes.
/// The returned `(vao, vbo)` pair must be kept alive for the lifetime of
/// the renderer.
///
/// # Safety
///
/// Caller must hold a valid, current `glow::Context`.
pub unsafe fn create_quad(
    gl: &glow::Context,
) -> Result<(glow::VertexArray, glow::Buffer), String> {
    let vao = gl
        .create_vertex_array()
        .map_err(|e| format!("create_vertex_array: {e}"))?;
    let vbo = gl
        .create_buffer()
        .map_err(|e| format!("create_buffer: {e}"))?;

    gl.bind_vertex_array(Some(vao));
    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

    // Reinterpret the f32 array as raw bytes for the buffer upload.
    let bytes = std::slice::from_raw_parts(
        QUAD_VERTS.as_ptr().cast::<u8>(),
        std::mem::size_of_val(&QUAD_VERTS),
    );
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);

    // Attribute 0 = a_position (layout(location=0) in vert.glsl): 2 × f32
    gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    gl.bind_vertex_array(None);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);

    Ok((vao, vbo))
}

/// Bind the program, source texture, and uniforms, then draw the full-screen quad.
///
/// `source_texture` may be `None` before the first image is uploaded; in that
/// case the sampler returns the default value (transparent black).
///
/// # Safety
///
/// Caller must hold a valid, current `glow::Context` and all handles must
/// still be live.
pub unsafe fn draw_frame(
    gl: &glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    source_texture: Option<glow::Texture>,
    uniform_locs: &UniformLocations,
    params: &ParamsSnapshot,
) {
    gl.clear_color(0.0, 0.0, 0.0, 1.0);
    gl.clear(glow::COLOR_BUFFER_BIT);
    gl.use_program(Some(program));

    // Bind source image to texture unit 0 (may be None before first upload).
    gl.active_texture(glow::TEXTURE0);
    gl.bind_texture(glow::TEXTURE_2D, source_texture);

    uniform_locs.upload(gl, params);

    gl.bind_vertex_array(Some(vao));
    gl.draw_arrays(glow::TRIANGLES, 0, 6);
    gl.bind_vertex_array(None);
}

