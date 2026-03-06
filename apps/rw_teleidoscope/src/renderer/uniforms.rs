//! Cache WebGL uniform locations and upload shader uniforms for each draw call.

use glow::HasContext;

/// Cached handles for every shader uniform location.
///
/// Querying uniform locations is an O(n) string scan inside the driver.
/// Caching them once at program-link time and reusing the handles every frame
/// avoids that cost entirely.
pub struct UniformLocations {
    /// `sampler2D u_image` — the source texture bound to unit 0.
    pub u_image: Option<glow::UniformLocation>,
}

impl UniformLocations {
    /// Query and cache all uniform locations from `program`.
    ///
    /// A location is `None` when the uniform is not present in the shader
    /// source (e.g. because it was optimised away), which is not an error.
    pub fn new(gl: &glow::Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                u_image: gl.get_uniform_location(program, "u_image"),
            }
        }
    }

    /// Upload all uniform values for the current draw call.
    ///
    /// For M3 this only sets `u_image = 0` (texture unit 0).  Later milestones
    /// will extend this method with the full `KaleidoscopeParams` snapshot.
    pub fn upload(&self, gl: &glow::Context) {
        unsafe {
            if let Some(loc) = &self.u_image {
                // Texture unit 0 holds the source image.
                gl.uniform_1_i32(Some(loc), 0);
            }
        }
    }
}

