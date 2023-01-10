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

vec4 gradient(
    vec4 direction,
    vec2 raw_position,
    vec4 offsets[2],
    vec4 colors[8]
) {
    vec2 start = direction.xy;
    vec2 end = direction.zw;

    vec2 v1 = vec2(end - start);
    vec2 v2 = vec2(raw_position - start);
    vec2 unit = normalize(v1);
    float coord_offset = dot(unit, v2) / length(v1);

    //if a gradient has a start/end stop that is identical, the mesh will have a transparent fill
    vec4 color;

    float offsets_arr[8];
    offsets_arr[0] = offsets[0].x;
    offsets_arr[1] = offsets[0].y;
    offsets_arr[2] = offsets[0].z;
    offsets_arr[3] = offsets[0].w;
    offsets_arr[4] = offsets[1].x;
    offsets_arr[5] = offsets[1].y;
    offsets_arr[6] = offsets[1].z;
    offsets_arr[7] = offsets[1].w;

    int last_index = 7;
    for (int i = 0; i <= 7; i++) {
        if (offsets_arr[i] > 1.0) {
            last_index = i - 1;
            break;
        }
    }

    for (int i = 0; i < last_index; i++) {
        float curr_offset = offsets_arr[i];
        float next_offset = offsets_arr[i+1];

        if (coord_offset <= offsets_arr[0]) {
            //current coordinate is before the first defined offset, set it to the start color
            color = colors[0];
        }

        if (curr_offset <= coord_offset && coord_offset <= next_offset) {
            //current fragment is between the current offset & the next one, interpolate colors
            color = mix(colors[i], colors[i+1], smoothstep(
                curr_offset,
                next_offset,
                coord_offset
            ));
        }

        if (coord_offset >= offsets_arr[last_index]) {
            //current coordinate is after the last defined offset, set it to the last color
            color = colors[last_index];
        }
    }

    return color;
}

in vec2 v_raw_position;
in vec4 v_colors[8];
in vec4 v_offsets[2];
in vec4 v_direction;

void main() {
    gl_FragColor = gradient(v_direction, v_raw_position, v_offsets, v_colors);
}
