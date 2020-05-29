#version 330

uniform float u_ScreenHeight;

in vec4 v_Color;
in vec4 v_BorderColor;
in vec2 v_Pos;
in vec2 v_Scale;
in float v_BorderRadius;
in float v_BorderWidth;

out vec4 o_Color;

float distance(in vec2 frag_coord, in vec2 position, in vec2 size, float radius)
{
    // TODO: Try SDF approach: https://www.shadertoy.com/view/wd3XRN
    vec2 inner_size = size - vec2(radius, radius) * 2.0;
    vec2 top_left = position + vec2(radius, radius);
    vec2 bottom_right = top_left + inner_size;

    vec2 top_left_distance = top_left - frag_coord;
    vec2 bottom_right_distance = frag_coord - bottom_right;

    vec2 distance = vec2(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );

    return sqrt(distance.x * distance.x + distance.y * distance.y);
}

void main() {
    vec4 mixed_color;

    vec2 fragCoord = vec2(gl_FragCoord.x, u_ScreenHeight - gl_FragCoord.y);

    // TODO: Remove branching (?)
    if(v_BorderWidth > 0) {
        float internal_border = max(v_BorderRadius - v_BorderWidth, 0.0);

        float internal_distance = distance(
            fragCoord,
            v_Pos + vec2(v_BorderWidth),
            v_Scale - vec2(v_BorderWidth * 2.0),
            internal_border
        );

        float border_mix = smoothstep(
            max(internal_border - 0.5, 0.0),
            internal_border + 0.5,
            internal_distance
        );

        mixed_color = mix(v_Color, v_BorderColor, border_mix);
    } else {
        mixed_color = v_Color;
    }

    float d = distance(
        fragCoord,
        v_Pos,
        v_Scale,
        v_BorderRadius
    );

    float radius_alpha =
        1.0 - smoothstep(max(v_BorderRadius - 0.5, 0.0), v_BorderRadius + 0.5, d);

    o_Color = vec4(mixed_color.xyz, mixed_color.w * radius_alpha);
}
