#version 450

layout(location = 0) in vec4 v_Color;
layout(location = 1) in vec4 v_BorderColor;
layout(location = 2) in vec2 v_Pos;
layout(location = 3) in vec2 v_Scale;
layout(location = 4) in float v_BorderRadius;
layout(location = 5) in float v_BorderWidth;

layout(location = 0) out vec4 o_Color;

float quadDistance(in vec2 frag_coord, in vec2 position, in vec2 size, float radius)
{
    vec2 inner_size = size - vec2(radius, radius) * 2.0;
    vec2 top_left = position + vec2(radius, radius);
    vec2 bottom_right = top_left + inner_size;

    vec2 top_left_distance = top_left - frag_coord;
    vec2 bottom_right_distance = frag_coord - bottom_right;

    vec2 distance = vec2(
        max(max(top_left_distance.x, bottom_right_distance.x), 0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0)
    );

    return sqrt(distance.x * distance.x + distance.y * distance.y);
}

vec4 quadColor(in vec4 bg_color, in vec4 frame_color, float radius, float frame_width, float d, float s)
{
    float inner_radius = radius - frame_width;
    float alpha = 1.0 - smoothstep(radius - s, radius + s, d);
    float mix_factor = smoothstep(inner_radius - s, inner_radius + s, d);
    vec4 c = mix(bg_color, frame_color, min(frame_width, mix_factor));
    return vec4(c.xyz, c.w * alpha);
}

void main() {
    // smoothens the edges of the rounded quad
    float sf = 0.5;

    float d = quadDistance(gl_FragCoord.xy, v_Pos, v_Scale, v_BorderRadius);
    o_Color = quadColor(v_Color, v_BorderColor, v_BorderRadius, v_BorderWidth, d, sf);
}
