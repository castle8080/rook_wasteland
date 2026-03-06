#version 300 es
precision highp float;

in vec2 v_uv;
out vec4 fragColor;

void main() {
    // Passthrough placeholder — outputs a solid dark colour.
    // Real kaleidoscope logic will be added in M4.
    fragColor = vec4(0.1, 0.1, 0.1, 1.0);
}
