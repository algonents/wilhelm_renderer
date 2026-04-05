#version 330 core
uniform vec4 geometryColor;
in vec4 vInstanceColor;
out vec4 FragColor;
void main()
{
    // Use per-instance color when provided (alpha > 0), otherwise fall back to uniform
    if (vInstanceColor.a > 0.0)
        FragColor = vInstanceColor;
    else
        FragColor = geometryColor;
}