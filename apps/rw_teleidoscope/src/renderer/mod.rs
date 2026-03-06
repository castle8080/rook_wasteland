//! WebGL 2 rendering pipeline.
//!
//! The `Renderer` struct owns all GPU resources (context, program, geometry)
//! and is the sole interface that `CanvasView` uses to draw frames.

pub mod context;
pub mod draw;
pub mod shader;
pub mod texture;
pub mod uniforms;

use glow::HasContext;

/// WebGL 2 renderer.
///
/// Owns all GPU resources.  Because `glow::Context` is `!Send`, the renderer
/// must **never** be placed inside a Leptos `RwSignal`.  Store it instead as
/// `Rc<RefCell<Option<Renderer>>>` inside the component that owns the canvas.
pub struct Renderer {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    /// Retained to prevent premature deletion; freed in `Drop`.
    vbo: glow::Buffer,
}

impl Renderer {
    /// Initialise the renderer for `canvas`.
    ///
    /// Obtains the WebGL 2 context, fetches and compiles the GLSL shaders
    /// from `assets/shaders/`, and uploads the static quad geometry.  Any
    /// error is returned as a human-readable string; the caller is expected
    /// to surface it via `console.error`.
    pub async fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, String> {
        let gl = context::get_context(canvas)?;
        let program = shader::create_program(&gl).await?;
        let (vao, vbo) = unsafe { draw::create_quad(&gl)? };
        Ok(Self {
            gl,
            program,
            vao,
            vbo,
        })
    }

    /// Draw one frame.
    ///
    /// For M2 this outputs the solid steampunk-brass colour defined in
    /// `frag.glsl`.  Texture sampling and uniforms are wired in later
    /// milestones.
    pub fn draw(&self) {
        unsafe { draw::draw_quad(&self.gl, self.program, self.vao) }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_program(self.program);
        }
    }
}

