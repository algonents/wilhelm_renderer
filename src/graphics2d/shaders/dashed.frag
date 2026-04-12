#version 330 core
uniform vec4 geometryColor;
uniform float u_dash_length;
uniform float u_gap_length;
in vec4 vInstanceColor;
in float vLineDist;
out vec4 FragColor;
void main()
{
    float cycle = u_dash_length + u_gap_length;
    float t = mod(vLineDist, cycle);
    if (t > u_dash_length)
        discard;

    // Use per-instance color when provided (alpha > 0), otherwise fall back to uniform
    if (vInstanceColor.a > 0.0)
        FragColor = vInstanceColor;
    else
        FragColor = geometryColor;
}
