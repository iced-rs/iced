#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif

// #includes

uniform float u_screen_height;

varying vec4 v_color;
varying vec4 v_border_color;
varying vec2 v_position;
varying vec2 v_size;
varying vec4 v_border_radius;
varying float v_border_width;

void main() {
    vec2 fragCoord = vec2(gl_FragCoord.x, u_screen_height - gl_FragCoord.y);

    float border_radius = selectBorderRadius(
        v_border_radius,
        fragCoord,
        (v_position + v_size * 0.5).xy
    );

    float internal_border = max(border_radius - v_border_width, 0.0);

    float internal_distance = fDistance(
        fragCoord,
        v_position + vec2(v_border_width),
        v_size - vec2(v_border_width * 2.0),
        internal_border
    );

    float border_mix = smoothstep(
        max(internal_border - 0.5, 0.0),
        internal_border + 0.5,
        internal_distance
    );

    vec4 mixed_color = mix(v_color, v_border_color, border_mix);

    float d = fDistance(
        fragCoord,
        v_position,
        v_size,
        border_radius
    );

    float radius_alpha = 1.0 - smoothstep(max(border_radius - 0.5, 0.0), border_radius + 0.5, d);

    gl_FragColor = vec4(mixed_color.xyz, mixed_color.w * radius_alpha);
}
