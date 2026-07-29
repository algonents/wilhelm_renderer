#version 330 core

in vec2 TexCoord;
out vec4 FragColor;

uniform sampler2D u_fontAtlas;
uniform vec4 u_color;

void main()
{
    // Signed distance field: 0.5 lies exactly on the glyph edge, >0.5 is inside.
    float d = texture(u_fontAtlas, TexCoord).r;
    float w = fwidth(d);
    float alpha = smoothstep(0.5 - w, 0.5 + w, d);
    FragColor = vec4(u_color.rgb, u_color.a * alpha);
}
