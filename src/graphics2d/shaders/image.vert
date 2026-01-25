#version 330 core

layout(location = 0) in vec2 aPos;

// Per-draw translation in screen/pixel coords
uniform vec2 u_screen_offset;
// Projection matrix
uniform mat4 u_Transform;
// Per-shape scale factor (default 1.0)
uniform float u_scale;

// Texture Coordinate: u, v
layout(location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
    vec2 p = aPos * u_scale + u_screen_offset;
    gl_Position = u_Transform * vec4(p, 0.0, 1.0);
    TexCoord = aTexCoord;
}
