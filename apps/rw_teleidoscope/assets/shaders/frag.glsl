#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

void main() {
    // Solid steampunk-brass colour: visually confirms WebGL is working.
    // Real kaleidoscope logic is added in M4.
    fragColor = vec4(0.545, 0.412, 0.078, 1.0);
}
