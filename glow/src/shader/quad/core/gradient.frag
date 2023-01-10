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

in vec4 v_colors[8];
in vec4 v_offsets[2];
in vec4 v_direction;
in vec4 v_position_and_size;
in vec4 v_border_color;
in vec4 v_border_radius;
in float v_border_width;

void main() {
    vec2 v_position = v_position_and_size.xy;
    vec2 v_size = v_position_and_size.zw;

    vec2 fragCoord = vec2(gl_FragCoord.x, u_screen_height - gl_FragCoord.y);

    vec4 mixed_color = gradient(v_direction, fragCoord, v_offsets, v_colors);

    float border_radius = selectBorderRadius(
        v_border_radius,
        fragCoord,
        (v_position + v_size * 0.5).xy
    );

    if (v_border_width > 0.0) {
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

        mixed_color = mix(mixed_color, v_border_color, border_mix);
    }

    float d = fDistance(
        fragCoord,
        v_position,
        v_size,
        border_radius
    );

    float radius_alpha = 1.0 - smoothstep(max(border_radius - 0.5, 0.0), border_radius + 0.5, d);

    gl_FragColor = vec4(mixed_color.xyz, mixed_color.w * radius_alpha);
}
