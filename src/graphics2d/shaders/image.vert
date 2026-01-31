#version 330 core

layout(location = 0) in vec2 aPos;

// Per-draw translation in screen/pixel coords
uniform vec2 u_screen_offset;
// Projection matrix
uniform mat4 u_Transform;
// Per-shape scale factor (default 1.0)
uniform float u_scale;
// Per-shape rotation in radians (default 0.0)
uniform float u_rotation;

// Texture Coordinate: u, v
layout(location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
    // Rotate around origin (local coordinates)
    float cos_r = cos(u_rotation);
    float sin_r = sin(u_rotation);
    vec2 rotated = vec2(
        aPos.x * cos_r - aPos.y * sin_r,
        aPos.x * sin_r + aPos.y * cos_r
    );
    // Scale, then translate
    vec2 p = rotated * u_scale + u_screen_offset;
    gl_Position = u_Transform * vec4(p, 0.0, 1.0);
    TexCoord = aTexCoord;
}
