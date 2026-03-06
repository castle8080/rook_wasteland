#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

uniform sampler2D u_image;
uniform int       u_segments;
uniform float     u_rotation;    // radians
uniform float     u_zoom;
uniform vec2      u_center;      // normalised 0.0–1.0

// M5 visual effects (0.0 = off, 1.0 = full effect)
uniform float     u_spiral;      // spiral twist intensity
uniform float     u_ripple;      // angular ripple intensity
uniform float     u_lens;        // barrel / lens distortion
uniform float     u_radial_fold; // concentric radial fold
uniform int       u_mobius;      // Möbius segment flip: 0=off, 1=on

const float PI  = 3.14159265358979323846;
const float TAU = 6.28318530717958647692;

void main() {
    // Map to polar coordinates centred on u_center.
    vec2  centered = v_uv - u_center;
    float r        = length(centered);
    float a        = atan(centered.y, centered.x) + u_rotation;

    // 1. Lens distortion (barrel warp): r = r / max(1 - lens*r*r, 0.001)
    if (u_lens > 0.0) {
        float denom = max(1.0 - u_lens * r * r, 0.001);
        r = r / denom;
    }

    // 2. Angular ripple: offset angle by ripple * sin(r * 20)
    if (u_ripple > 0.0) {
        a += u_ripple * sin(r * 20.0);
    }

    // 3. Spiral twist: offset angle by spiral * r * TAU
    a += u_spiral * r * TAU;

    // 4. Mirror-fold angle into [0, PI/segments].
    //    Record segment index before folding (needed for Möbius).
    float seg_angle = PI / float(u_segments);
    float two_seg   = 2.0 * seg_angle;
    float seg_idx   = floor(a / two_seg);   // integer segment index (as float)
    a = mod(a, two_seg);
    if (a > seg_angle) {
        a = two_seg - a;
    }

    // 5. Möbius flip: for odd segment indices, negate r so the UV sampling
    //    is inverted relative to the centre, producing alternating flips.
    if (u_mobius != 0 && mod(seg_idx, 2.0) > 0.5) {
        r = -r;
    }

    // 6. Radial folding: abs(mod(r*(1 + fold*4), 2) - 1).
    //    Gated because the formula is not an identity at fold=0.
    if (u_radial_fold > 0.0) {
        r = abs(mod(r * (1.0 + u_radial_fold * 4.0), 2.0) - 1.0);
    }

    // Reconstruct Cartesian UV, apply zoom scale, wrap-sample the texture.
    vec2 sample_uv = vec2(r * cos(a), r * sin(a)) * u_zoom + u_center;
    fragColor = texture(u_image, fract(sample_uv));
}
