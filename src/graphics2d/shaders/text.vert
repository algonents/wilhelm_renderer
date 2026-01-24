#version 330 core

layout(location = 0) in vec2 aPos;
layout(location = 1) in vec2 aTexCoord;

uniform vec2 u_screen_offset;
uniform mat4 u_Transform;

out vec2 TexCoord;

void main() {
    vec2 p = aPos + u_screen_offset;
    gl_Position = u_Transform * vec4(p, 0.0, 1.0);
    TexCoord = aTexCoord;
}
