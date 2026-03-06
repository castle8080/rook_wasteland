//! Upload image pixel data to a WebGL 2 texture.

use glow::HasContext;

/// Create a new WebGL 2 texture from `image_data`, upload the RGBA pixels,
/// configure sampling parameters, and return the texture handle.
///
/// The texture is:
/// - bound to `TEXTURE0` on return (the caller may rebind as needed)
/// - wrapped with `CLAMP_TO_EDGE` on both axes
/// - filtered with `LINEAR` for both minification and magnification
///
/// If the caller already holds a texture for this purpose, they must delete it
/// before calling this function to avoid a GPU memory leak.
pub fn upload_image_data(
    gl: &glow::Context,
    image_data: &web_sys::ImageData,
) -> Result<glow::Texture, String> {
    let width = image_data.width() as i32;
    let height = image_data.height() as i32;
    // Copy the pixel bytes out of the JS Uint8ClampedArray into a Vec<u8>.
    // 800×800×4 ≈ 2.5 MB — acceptable for a one-shot upload.
    let raw: Vec<u8> = image_data.data().to_vec();

    unsafe {
        let texture = gl
            .create_texture()
            .map_err(|e| format!("create_texture: {e}"))?;

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));

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

        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as i32,
            width,
            height,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(raw.as_slice()),
        );

        gl.bind_texture(glow::TEXTURE_2D, None);

        Ok(texture)
    }
}

