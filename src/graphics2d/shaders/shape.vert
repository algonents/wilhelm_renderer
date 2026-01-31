#version 330 core

uniform mat4 u_Transform;                     // projection matrix
uniform vec2 u_screen_offset;                 // single-shape translation (uniform). Forced to 0 when instancing
uniform float u_scale;                        // per-shape scale factor (default 1.0)
uniform float u_rotation;                     // per-shape rotation in radians (default 0.0)

layout (location = 0) in vec2 aPos;           // mesh-local vertex
layout (location = 1) in vec2 aInstanceXY;    // optional; if disabled => (0,0)
layout (location = 2) in vec4 aInstanceColor; // optional; if disabled => (0,0,0,0)

out vec4 vInstanceColor;

void main() {
    // Rotate around origin (local coordinates)
    float cos_r = cos(u_rotation);
    float sin_r = sin(u_rotation);
    vec2 rotated = vec2(
        aPos.x * cos_r - aPos.y * sin_r,
        aPos.x * sin_r + aPos.y * cos_r
    );
    // Scale, then translate
    vec2 p = rotated * u_scale + u_screen_offset + aInstanceXY;
    gl_Position = u_Transform * vec4(p, 0.0, 1.0);
    vInstanceColor = aInstanceColor;
}