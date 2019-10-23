#version 450

layout(location = 0) in vec4 v_Color;
layout(location = 1) in vec2 v_Pos;
layout(location = 2) in vec2 v_Scale;
layout(location = 3) in flat uint v_BorderRadius;

layout(location = 0) out vec4 o_Color;

float rounded(in vec2 frag_coord, in vec2 position, in vec2 size, float radius, float s)
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

    float d = sqrt(distance.x * distance.x + distance.y * distance.y);

    return 1.0 - smoothstep(radius - s, radius + s, d);
}

void main() {
    float radius_alpha = 1.0;

    if(v_BorderRadius > 0.0) {
        radius_alpha = rounded(gl_FragCoord.xy, v_Pos, v_Scale, v_BorderRadius, 0.5);
    }

    o_Color = vec4(v_Color.xyz, v_Color.w * radius_alpha);
}
