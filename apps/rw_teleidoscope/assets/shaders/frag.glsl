#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_image;

void main() {
    // Passthrough: sample the source image at the interpolated UV coordinate.
    // Kaleidoscope transforms are added from M4 onwards.
    fragColor = texture(u_image, v_uv);
}
