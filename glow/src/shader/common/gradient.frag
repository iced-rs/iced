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

uniform vec4 gradient_direction;
uniform int color_stops_size;
// GLSL does not support dynamically sized arrays without SSBOs so this is capped to 16 stops
//stored as color(vec4) -> offset(vec4) sequentially;
uniform vec4 color_stops[32];

//TODO: rewrite without branching to make ALUs happy
void main() {
    vec2 start = gradient_direction.xy;
    vec2 end = gradient_direction.zw;
    vec2 gradient_vec = vec2(end - start);
    vec2 current_vec = vec2(raw_position.xy - start);
    vec2 unit = normalize(gradient_vec);
    float coord_offset = dot(unit, current_vec) / length(gradient_vec);
    //if a gradient has a start/end stop that is identical, the mesh will have a transparent fill
    gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);

    float min_offset = color_stops[1].x;
    float max_offset = color_stops[color_stops_size - 1].x;

    for (int i = 0; i < color_stops_size - 2; i += 2) {
        float curr_offset = color_stops[i+1].x;
        float next_offset = color_stops[i+3].x;

        if (coord_offset <= min_offset) {
            //current coordinate is before the first defined offset, set it to the start color
            gl_FragColor = color_stops[0];
        }

        if (curr_offset <= coord_offset && coord_offset <= next_offset) {
            //current fragment is between the current offset processing & the next one, interpolate colors
            gl_FragColor = mix(color_stops[i], color_stops[i+2], smoothstep(
                curr_offset,
                next_offset,
                coord_offset
            ));
        }

        if (coord_offset >= max_offset) {
            //current coordinate is before the last defined offset, set it to the last color
            gl_FragColor = color_stops[color_stops_size - 2];
        }
    }
}
