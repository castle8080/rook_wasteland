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

// M6 color transforms
uniform float     u_hue_shift;   // hue rotation in degrees (0–360)
uniform float     u_saturation;  // saturation multiplier (0–2; 1=unchanged)
uniform float     u_brightness;  // brightness multiplier (0–2; 1=unchanged)
uniform int       u_posterize;   // posterize levels (0=off, 2–16=active)
uniform int       u_invert;      // colour inversion: 0=off, 1=on

const float PI  = 3.14159265358979323846;
const float TAU = 6.28318530717958647692;

// ---------------------------------------------------------------------------
// M6 color transform helpers
// ---------------------------------------------------------------------------

/// Convert a linear RGB colour to HSV.
/// Returns vec3(hue_degrees [0,360), saturation [0,1], value [0,1]).
vec3 rgb_to_hsv(vec3 c) {
    float cmax  = max(c.r, max(c.g, c.b));
    float cmin  = min(c.r, min(c.g, c.b));
    float delta = cmax - cmin;

    float h = 0.0;
    if (delta > 0.0) {
        if (cmax == c.r) {
            h = 60.0 * mod((c.g - c.b) / delta, 6.0);
        } else if (cmax == c.g) {
            h = 60.0 * ((c.b - c.r) / delta + 2.0);
        } else {
            h = 60.0 * ((c.r - c.g) / delta + 4.0);
        }
    }

    float s = (cmax > 0.0) ? (delta / cmax) : 0.0;
    return vec3(h, s, cmax);
}

/// Convert HSV (hue_degrees, saturation, value) back to linear RGB.
vec3 hsv_to_rgb(vec3 c) {
    float h  = c.x;
    float s  = c.y;
    float v  = c.z;
    float f  = mod(h / 60.0, 6.0);
    float i  = floor(f);
    float ff = f - i;
    float p  = v * (1.0 - s);
    float q  = v * (1.0 - s * ff);
    float t  = v * (1.0 - s * (1.0 - ff));

    if (i < 0.5) return vec3(v, t, p);
    if (i < 1.5) return vec3(q, v, p);
    if (i < 2.5) return vec3(p, v, t);
    if (i < 3.5) return vec3(p, q, v);
    if (i < 4.5) return vec3(t, p, v);
    return vec3(v, p, q);
}

/// Rotate the hue of `colour` by `degrees`.
vec4 hue_rotate(vec4 colour, float degrees) {
    if (degrees == 0.0) return colour;
    vec3 hsv = rgb_to_hsv(colour.rgb);
    hsv.x = mod(hsv.x + degrees, 360.0);
    return vec4(hsv_to_rgb(hsv), colour.a);
}

/// Adjust saturation by interpolating between greyscale and full colour.
/// `amount` 0 = greyscale, 1 = unchanged, 2 = hyper-saturated.
vec3 saturate_rgb(vec3 colour, float amount) {
    // Rec. 709 luminance coefficients
    float lum = dot(colour, vec3(0.2126, 0.7152, 0.0722));
    return mix(vec3(lum), colour, amount);
}

/// Quantise each channel to `levels` discrete steps.
/// Input is clamped to [0,1] first to avoid artefacts from HDR-ish values.
vec4 posterize(vec4 colour, int levels) {
    float lev = float(levels);
    return vec4(floor(clamp(colour.rgb, 0.0, 1.0) * lev) / lev, colour.a);
}

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
    vec4 colour = texture(u_image, fract(sample_uv));

    // M6 color transforms — applied in order: hue → sat → brightness → posterize → invert
    colour     = hue_rotate(colour, u_hue_shift);
    colour.rgb = saturate_rgb(colour.rgb, u_saturation);
    colour.rgb *= u_brightness;
    if (u_posterize > 1) colour = posterize(colour, u_posterize);
    if (u_invert != 0)   colour.rgb = 1.0 - colour.rgb;

    fragColor = colour;
}
