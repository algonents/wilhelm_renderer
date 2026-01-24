#version 330 core

in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D u_fontAtlas;
uniform vec4 u_color;

void main() {
    // Sample the red channel from the font atlas (grayscale glyph)
    float alpha = texture(u_fontAtlas, TexCoord).r;
    FragColor = vec4(u_color.rgb, u_color.a * alpha);
}
