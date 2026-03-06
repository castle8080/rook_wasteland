#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_image;
uniform int       u_segments;
uniform float     u_rotation;   // radians
uniform float     u_zoom;
uniform vec2      u_center;     // normalised 0.0–1.0

const float PI = 3.14159265358979323846;

void main() {
    // Map to polar coordinates centred on u_center.
    vec2  centered  = v_uv - u_center;
    float r         = length(centered);
    float a         = atan(centered.y, centered.x) + u_rotation;

    // Mirror-fold angle into the fundamental domain [0, PI/segments].
    float seg_angle = PI / float(u_segments);
    a = mod(a, 2.0 * seg_angle);
    if (a > seg_angle) {
        a = 2.0 * seg_angle - a;
    }

    // Reconstruct Cartesian UV, apply zoom scale, wrap-sample the texture.
    vec2 sample_uv = vec2(r * cos(a), r * sin(a)) * u_zoom + u_center;
    fragColor = texture(u_image, fract(sample_uv));
}
