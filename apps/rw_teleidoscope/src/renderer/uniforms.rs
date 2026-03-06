//! Cache WebGL uniform locations and upload shader uniforms for each draw call.

use glow::HasContext;

use crate::state::ParamsSnapshot;

/// Cached handles for every shader uniform location.
///
/// Querying uniform locations is an O(n) string scan inside the driver.
/// Caching them once at program-link time and reusing the handles every frame
/// avoids that cost entirely.
pub struct UniformLocations {
    /// `sampler2D u_image` — the source texture bound to unit 0.
    pub u_image:       Option<glow::UniformLocation>,
    /// `int u_segments` — mirror segment count (2–10).
    pub u_segments:    Option<glow::UniformLocation>,
    /// `float u_rotation` — pattern rotation in radians.
    pub u_rotation:    Option<glow::UniformLocation>,
    /// `float u_zoom` — source sampling scale.
    pub u_zoom:        Option<glow::UniformLocation>,
    /// `vec2 u_center` — centre of symmetry, normalised 0..1.
    pub u_center:      Option<glow::UniformLocation>,
    // M5 visual effects
    /// `float u_spiral` — spiral twist intensity (0–1).
    pub u_spiral:      Option<glow::UniformLocation>,
    /// `float u_ripple` — angular ripple intensity (0–1).
    pub u_ripple:      Option<glow::UniformLocation>,
    /// `float u_lens` — barrel / lens distortion (0–1).
    pub u_lens:        Option<glow::UniformLocation>,
    /// `float u_radial_fold` — concentric radial fold (0–1).
    pub u_radial_fold: Option<glow::UniformLocation>,
    /// `int u_mobius` — Möbius segment flip (0=off, 1=on).
    pub u_mobius:      Option<glow::UniformLocation>,
}

impl UniformLocations {
    /// Query and cache all uniform locations from `program`.
    ///
    /// A location is `None` when the uniform is not present in the shader
    /// source (e.g. because it was optimised away), which is not an error.
    pub fn new(gl: &glow::Context, program: glow::Program) -> Self {
        unsafe {
            Self {
                u_image:       gl.get_uniform_location(program, "u_image"),
                u_segments:    gl.get_uniform_location(program, "u_segments"),
                u_rotation:    gl.get_uniform_location(program, "u_rotation"),
                u_zoom:        gl.get_uniform_location(program, "u_zoom"),
                u_center:      gl.get_uniform_location(program, "u_center"),
                u_spiral:      gl.get_uniform_location(program, "u_spiral"),
                u_ripple:      gl.get_uniform_location(program, "u_ripple"),
                u_lens:        gl.get_uniform_location(program, "u_lens"),
                u_radial_fold: gl.get_uniform_location(program, "u_radial_fold"),
                u_mobius:      gl.get_uniform_location(program, "u_mobius"),
            }
        }
    }

    /// Upload all uniform values for the current draw call.
    pub fn upload(&self, gl: &glow::Context, params: &ParamsSnapshot) {
        unsafe {
            if let Some(loc) = &self.u_image {
                // Texture unit 0 holds the source image.
                gl.uniform_1_i32(Some(loc), 0);
            }
            if let Some(loc) = &self.u_segments {
                gl.uniform_1_i32(Some(loc), params.segments as i32);
            }
            if let Some(loc) = &self.u_rotation {
                // Signal stores degrees; shader expects radians.
                gl.uniform_1_f32(Some(loc), params.rotation.to_radians());
            }
            if let Some(loc) = &self.u_zoom {
                gl.uniform_1_f32(Some(loc), params.zoom);
            }
            if let Some(loc) = &self.u_center {
                gl.uniform_2_f32(Some(loc), params.center.0, params.center.1);
            }
            if let Some(loc) = &self.u_spiral {
                gl.uniform_1_f32(Some(loc), params.spiral);
            }
            if let Some(loc) = &self.u_ripple {
                gl.uniform_1_f32(Some(loc), params.ripple);
            }
            if let Some(loc) = &self.u_lens {
                gl.uniform_1_f32(Some(loc), params.lens);
            }
            if let Some(loc) = &self.u_radial_fold {
                gl.uniform_1_f32(Some(loc), params.radial_fold);
            }
            if let Some(loc) = &self.u_mobius {
                gl.uniform_1_i32(Some(loc), i32::from(params.mobius));
            }
        }
    }
}

