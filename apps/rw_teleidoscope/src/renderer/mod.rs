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
// FBO helper
// ---------------------------------------------------------------------------

/// Allocate a framebuffer and two 800×800 RGBA8 ping-pong textures for
/// recursive-reflection multi-pass rendering.
///
/// # Safety
///
/// Caller must hold a valid, current `glow::Context`.
unsafe fn create_fbo(
    gl: &glow::Context,
) -> Result<(glow::Framebuffer, [glow::Texture; 2]), String> {
    const FBO_SIZE: i32 = 800;

    let fbo = gl
        .create_framebuffer()
        .map_err(|e| format!("create_framebuffer: {e}"))?;

    let tex0 = gl
        .create_texture()
        .map_err(|e| format!("create_texture fbo0: {e}"))?;
    let tex1 = gl
        .create_texture()
        .map_err(|e| format!("create_texture fbo1: {e}"))?;

    for tex in [tex0, tex1] {
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            FBO_SIZE,
            FBO_SIZE,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
    }

    gl.bind_texture(glow::TEXTURE_2D, None);
    Ok((fbo, [tex0, tex1]))
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
    /// Single framebuffer used for recursive-reflection multi-pass rendering.
    fbo: glow::Framebuffer,
    /// Two ping-pong textures (800×800 RGBA8) attached to `fbo` alternately.
    /// Index 0 receives the first off-screen pass; subsequent passes alternate.
    fbo_textures: [glow::Texture; 2],
}

impl Renderer {
    /// Initialise the renderer for `canvas`.
    ///
    /// Obtains the WebGL 2 context, compiles the embedded GLSL shaders,
    /// uploads the static quad geometry, caches uniform locations, and
    /// allocates the FBO ping-pong textures for recursive reflection.
    /// Any error is returned as a human-readable string.
    pub fn new(canvas: &web_sys::HtmlCanvasElement) -> Result<Self, String> {
        let gl = context::get_context(canvas)?;
        let program = shader::create_program(&gl)?;
        let uniform_locs = UniformLocations::new(&gl, program);
        let (vao, vbo) = unsafe { draw::create_quad(&gl)? };
        let (fbo, fbo_textures) = unsafe { create_fbo(&gl)? };
        Ok(Self {
            gl,
            program,
            vao,
            vbo,
            source_texture: None,
            uniform_locs,
            fbo,
            fbo_textures,
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
    ///
    /// When `params.recursive_depth == 0` a single pass renders directly to the
    /// canvas default framebuffer.  For depth ≥ 1, the first pass renders to an
    /// off-screen FBO texture; subsequent passes ping-pong between two FBO
    /// textures until the requested depth is reached; the final pass renders to
    /// the canvas.
    pub fn draw(&self, params: &ParamsSnapshot) {
        unsafe {
            if params.recursive_depth == 0 {
                // Normal single-pass render to the canvas.
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                draw::draw_frame(
                    &self.gl,
                    self.program,
                    self.vao,
                    self.source_texture,
                    &self.uniform_locs,
                    params,
                );
            } else {
                // Multi-pass recursive reflection.
                // Pass 0: source_texture → fbo_textures[0]
                self.bind_fbo_texture(0);
                draw::draw_frame(
                    &self.gl,
                    self.program,
                    self.vao,
                    self.source_texture,
                    &self.uniform_locs,
                    params,
                );

                // Passes 1..depth-1: fbo_textures[src] → fbo_textures[dst]
                for i in 1..params.recursive_depth {
                    let src = ((i - 1) % 2) as usize;
                    let dst = (i % 2) as usize;
                    self.bind_fbo_texture(dst);
                    draw::draw_frame(
                        &self.gl,
                        self.program,
                        self.vao,
                        Some(self.fbo_textures[src]),
                        &self.uniform_locs,
                        params,
                    );
                }

                // Final pass: last fbo texture → canvas default framebuffer.
                let last = ((params.recursive_depth - 1) % 2) as usize;
                self.gl.bind_framebuffer(glow::FRAMEBUFFER, None);
                draw::draw_frame(
                    &self.gl,
                    self.program,
                    self.vao,
                    Some(self.fbo_textures[last]),
                    &self.uniform_locs,
                    params,
                );
            }
        }
    }

    /// Bind the FBO and attach `fbo_textures[idx]` as the colour attachment.
    ///
    /// # Safety
    /// Caller must hold a valid `glow::Context` (guaranteed by `&self`).
    unsafe fn bind_fbo_texture(&self, idx: usize) {
        self.gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));
        self.gl.framebuffer_texture_2d(
            glow::FRAMEBUFFER,
            glow::COLOR_ATTACHMENT0,
            glow::TEXTURE_2D,
            Some(self.fbo_textures[idx]),
            0,
        );
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(tex) = self.source_texture {
                self.gl.delete_texture(tex);
            }
            for tex in self.fbo_textures {
                self.gl.delete_texture(tex);
            }
            self.gl.delete_framebuffer(self.fbo);
            self.gl.delete_buffer(self.vbo);
            self.gl.delete_vertex_array(self.vao);
            self.gl.delete_program(self.program);
        }
    }
}

