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
uniform uint color_stops_size;
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

    for (uint i = 0; i < color_stops_size - 2; i += 2) {
        vec4 color = color_stops[i];
        float offset = color_stops[i+1].x;

        vec4 next_color = color_stops[i+2];
        float next_offset = color_stops[i+3].x;

        if (offset <= coord_offset && coord_offset <= next_offset) {
            fragColor = mix(color, next_color, smoothstep(
                offset,
                next_offset,
                coord_offset
            ));
        } else if (coord_offset < color_stops[1].x) {
            fragColor = color_stops[0];
        } else if (coord_offset > color_stops[color_stops_size - 1].x) {
            fragColor = color_stops[color_stops_size - 2];
        }
    }
}