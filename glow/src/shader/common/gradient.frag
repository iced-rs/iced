// GLSL does not support dynamically sized arrays without SSBOs
#define MAX_STOPS 64

#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif

#ifdef HIGHER_THAN_300
layout (location = 0) out vec4 fragColor;
#define gl_FragColor fragColor
#endif

in vec2 raw_position;

uniform vec2 gradient_start;
uniform vec2 gradient_end;

uniform uint color_stops_size;
uniform float color_stop_offsets[MAX_STOPS];
uniform vec4 color_stop_colors[MAX_STOPS];

void main() {
    vec2 gradient_vec = vec2(gradient_end - gradient_start);
    vec2 current_vec = vec2(raw_position.xy - gradient_start);
    vec2 unit = normalize(gradient_vec);
    float coord_offset = dot(unit, current_vec) / length(gradient_vec);

    for (uint i = 0; i < color_stops_size - 1; i++) {
        float stop_offset = color_stop_offsets[i];
        float next_stop_offset = color_stop_offsets[i + 1];

        if (stop_offset <= coord_offset && coord_offset <= next_stop_offset) {
            fragColor = mix(color_stop_colors[i], color_stop_colors[i+1], smoothstep(
                stop_offset,
                next_stop_offset,
                coord_offset
            ));
        } else if (coord_offset < color_stop_offsets[0]) {
            fragColor = color_stop_colors[0];
        } else if (coord_offset > color_stop_offsets[color_stops_size - 1]) {
            fragColor = color_stop_colors[color_stops_size - 1];
        }
    }
}