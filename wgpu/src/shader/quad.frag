#version 450

layout(location = 0) in vec4 v_Color;
layout(location = 1) in vec4 v_BorderColor;
layout(location = 2) in vec2 v_Pos;
layout(location = 3) in vec2 v_Scale;
layout(location = 4) in float v_BorderRadius;
layout(location = 5) in float v_BorderWidth;

layout(location = 0) out vec4 o_Color;

float sdBox( in vec2 p, in vec2 b )
{
    vec2 d = abs(p)-b;
    return length(max(d,vec2(0))) + min(max(d.x,d.y),0.0);
}

float sdRoundedBox(in vec2 p, in vec2 b, in float r)
{
    return sdBox(p, b - r) - r;
}

float sdAnnularRoundedBox(in vec2 p, in vec2 b, in float r, in float w)
{
    return abs(sdRoundedBox(p, b - w, r - w)) - w;
}

float drawRoundedBox(in vec2 coord, in vec2 pos, in vec2 size, in float r, in float w)
{
    vec2 p = pos + w;
    vec2 s = size - 2.0 * w;
    return 1.0 - clamp(sdRoundedBox(coord + 0.5 - s / 2.0 - p, s/ 2.0, r - w), 0.0, 1.0);
}

float drawRoundedFrame(in vec2 coord, in vec2 pos, in vec2 size, in float r, in float w)
{
    return min(w, 1.0 - clamp(sdAnnularRoundedBox(coord + 0.5 - size / 2.0 - pos, size / 2.0, r, w / 2.0), 0.0, 1.0));
}

void main() {
    vec4 color = vec4(0);

    // border
    float frame = drawRoundedFrame(gl_FragCoord.xy, v_Pos, v_Scale, v_BorderRadius, v_BorderWidth);
    color = mix(color, v_BorderColor, frame);

    // content
    float content = drawRoundedBox(gl_FragCoord.xy, v_Pos, v_Scale, v_BorderRadius, v_BorderWidth);
    color = mix(color, v_Color, content);
    
    o_Color = color;
}
