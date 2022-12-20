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

// #includes

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
