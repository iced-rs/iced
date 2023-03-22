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

float selectBorderRadius(vec4 radi, vec2 position, vec2 center)
{
    float rx = radi.x;
    float ry = radi.y;
    rx = position.x > center.x ? radi.y : radi.x;
    ry = position.x > center.x ? radi.z : radi.w;
    rx = position.y > center.y ? ry : rx;
    return rx;
}

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

uniform float u_screen_height;

in vec4 v_color;
in vec4 v_border_color;
in vec2 v_position;
in vec2 v_scale;
in vec4 v_border_radius;
in float v_border_width;

void main() {
    vec4 mixed_color;

    vec2 fragCoord = vec2(gl_FragCoord.x, u_screen_height - gl_FragCoord.y);

    float border_radius = selectBorderRadius(
        v_border_radius,
        fragCoord,
        (v_position + v_scale * 0.5).xy
    );

    if (v_border_width > 0.0) {
        float internal_border = max(border_radius - v_border_width, 0.0);

        float internal_distance = fDistance(
            fragCoord,
            v_position + vec2(v_border_width),
            v_scale - vec2(v_border_width * 2.0),
            internal_border
        );

        float border_mix = smoothstep(
            max(internal_border - 0.5, 0.0),
            internal_border + 0.5,
            internal_distance
        );

        mixed_color = mix(v_color, v_border_color, border_mix);
    } else {
        mixed_color = v_color;
    }

    float d = fDistance(
        fragCoord,
        v_position,
        v_scale,
        border_radius
    );

    float radius_alpha = 1.0 - smoothstep(max(border_radius - 0.5, 0.0), border_radius + 0.5, d);

    gl_FragColor = vec4(mixed_color.xyz, mixed_color.w * radius_alpha);
}
