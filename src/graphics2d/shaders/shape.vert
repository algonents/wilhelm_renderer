#version 330 core

uniform mat4 u_Transform;                     // projection matrix
uniform vec2 u_screen_offset;                 // single-shape translation (uniform). Forced to 0 when instancing
uniform float u_scale;                        // per-shape scale factor (default 1.0)

layout (location = 0) in vec2 aPos;           // mesh-local vertex
layout (location = 1) in vec2 aInstanceXY;    // optional; if disabled => (0,0)
layout (location = 2) in vec4 aInstanceColor; // optional; if disabled => (0,0,0,0)

out vec4 vInstanceColor;

void main() {
    vec2 p = aPos * u_scale + u_screen_offset + aInstanceXY;
    gl_Position = u_Transform * vec4(p, 0.0, 1.0);
    vInstanceColor = aInstanceColor;
}