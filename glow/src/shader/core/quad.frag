#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif

#ifdef HIGHER_THAN_300
out vec4 fragColor;
#define gl_FragColor fragColor
#endif

uniform float u_ScreenHeight;

in vec4 v_Color;
in vec4 v_BorderColor;
in vec2 v_Pos;
in vec2 v_Scale;
in vec4 v_BorderRadius;
in float v_BorderWidth;

float fDistance(vec2 frag_coord, vec2 position, vec2 size, float radius)
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

float selectBorderRadius(vec4 radi, vec2 position, vec2 center)
{
    float rx = radi.x;
    float ry = radi.y;
    rx = position.x > center.x ? radi.y : radi.x;
    ry = position.x > center.x ? radi.z : radi.w;
    rx = position.y > center.y ? ry : rx;
    return rx;
}

void main() {
    vec4 mixed_color;

    vec2 fragCoord = vec2(gl_FragCoord.x, u_ScreenHeight - gl_FragCoord.y);

    float borderRadius = selectBorderRadius(
        v_BorderRadius,
        fragCoord,
        (v_Pos + v_Scale * 0.5).xy
    );

    // TODO: Remove branching (?)
    if(v_BorderWidth > 0.0) {
        float internal_border = max(borderRadius - v_BorderWidth, 0.0);

        float internal_distance = fDistance(
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

    float d = fDistance(
        fragCoord,
        v_Pos,
        v_Scale,
        borderRadius
    );

    float radius_alpha =
        1.0 - smoothstep(max(borderRadius - 0.5, 0.0), borderRadius + 0.5, d);

    gl_FragColor = vec4(mixed_color.xyz, mixed_color.w * radius_alpha);
}
