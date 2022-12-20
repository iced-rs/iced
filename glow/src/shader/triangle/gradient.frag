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

in vec2 v_raw_position;
in vec4 v_colors[8];
in vec4 v_offsets[2];
in vec4 v_direction;

void main() {
    gl_FragColor = gradient(v_direction, v_raw_position, v_offsets, v_colors);
}
