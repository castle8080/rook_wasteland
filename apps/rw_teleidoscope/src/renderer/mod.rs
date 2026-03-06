//! WebGL 2 rendering pipeline.
//!
//! The `Renderer` struct owns all GPU resources (context, program, geometry,
//! textures) and is the sole interface for drawing frames.
//!
//! Because `glow::Context` is `!Send + !Sync`, the renderer **cannot** be
//! placed inside a Leptos `RwSignal` (which requires `Send + Sync`).  It is
//! stored instead as a thread-local singleton and accessed via the free
//! functions [`set_renderer`], [`with_renderer`], [`with_renderer_mut`], and
//! [`draw`].

pub mod context;
pub mod draw;
pub mod shader;
pub mod texture;
pub mod uniforms;

use std::cell::RefCell;

use glow::HasContext;

use uniforms::UniformLocations;

use crate::state::ParamsSnapshot;

// ---------------------------------------------------------------------------
// Thread-local singleton
// ---------------------------------------------------------------------------

thread_local! {
    /// The single `Renderer` instance for this thread (main WASM thread).
    static RENDERER: RefCell<Option<Renderer>> = const { RefCell::new(None) };
}

/// Store `renderer` as the active singleton for this thread.
///
/// Any previously stored renderer is dropped (GPU resources freed).
pub fn set_renderer(renderer: Renderer) {
    RENDERER.with(|r| *r.borrow_mut() = Some(renderer));
}

/// Returns `true` if a renderer has been stored with [`set_renderer`].
pub fn is_initialized() -> bool {
    RENDERER.with(|r| r.borrow().is_some())
}

/// Call `f` with an immutable reference to the renderer.
///
/// Returns `None` if the renderer has not yet been initialised.
pub fn with_renderer<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Renderer) -> R,
{
    RENDERER.with(|r| r.borrow().as_ref().map(f))
}

/// Call `f` with a mutable reference to the renderer.
///
/// Returns `None` if the renderer has not yet been initialised.
pub fn with_renderer_mut<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Renderer) -> R,
{
    RENDERER.with(|r| r.borrow_mut().as_mut().map(f))
}

/// Issue a draw call on the global renderer.
///
/// This is a no-op if the renderer has not yet been initialised, which can
/// happen when the reactive effect fires before the canvas NodeRef resolves.
pub fn draw(params: &ParamsSnapshot) {
    with_renderer(|r| r.draw(params));
}

// ---------------------------------------------------------------------------
// Renderer struct
// ---------------------------------------------------------------------------

/// WebGL 2 renderer.
///
/// Owns all GPU resources.  Access via the module-level free functions rather
/// than constructing directly in component code; doing so avoids `!Send + !Sync`
/// conflicts with the Leptos signal system.
pub struct Renderer {
    gl: glow::Context,
    program: glow::Program,
    vao: glow::VertexArray,
    /// Retained to prevent premature deletion; freed in `Drop`.
    vbo: glow::Buffer,
    /// The uploaded source image texture, or `None` before first upload.
    source_texture: Option<glow::Texture>,
    /// Cached uniform location handles (populated at program-link time).
    uniform_locs: UniformLocations,
}

impl Renderer {
    /// Initialise the renderer for `canvas`.
    ///
    /// Obtains the WebGL 2 context, compiles the embedded GLSL shaders,
    /// uploads the static quad geometry, and caches uniform locations.
    /// Any error is returned as a human-readable string.
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, String> {
        let gl = context::get_context(canvas)?;
        let program = shader::create_program(&gl)?;
        let uniform_locs = UniformLocations::new(&gl, program);
        let (vao, vbo) = unsafe { draw::create_quad(&gl)? };
        Ok(Self {
            gl,
            program,
            vao,
            vbo,
            source_texture: None,
            uniform_locs,
        })
    }

    /// Upload `image_data` as the source texture (texture unit 0).
    ///
    /// Replaces any previously uploaded texture; the old GPU object is deleted
    /// first to avoid leaking GPU memory.  `image_data` must be 800 × 800 RGBA;
    /// use [`crate::utils::resize_to_800`] before calling this.
    pub fn upload_image(&mut self, image_data: &web_sys::ImageData) {
        // Delete old texture before allocating a new one.
        if let Some(old) = self.source_texture.take() {
            unsafe { self.gl.delete_texture(old) };
        }
        match texture::upload_image_data(&self.gl, image_data) {
            Ok(tex) => {
                self.source_texture = Some(tex);
            }
            Err(e) => {
                web_sys::console::error_1(&e.into());
            }
        }
    }

    /// Draw one frame using the current source texture and uniforms.
    pub fn draw(&self, params: &ParamsSnapshot) {
        unsafe {
            draw::draw_frame(
                &self.gl,
                self.program,
                self.vao,
                self.source_texture,
                &self.uniform_locs,
                params,
            );
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(tex) = self.source_texture {
                self.gl.delete_texture(tex);
            }
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_program(self.program);
        }
    }
}

