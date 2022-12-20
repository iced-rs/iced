uniform mat4 u_transform;

in vec2 i_position;
in vec4 i_colors_1;
in vec4 i_colors_2;
in vec4 i_colors_3;
in vec4 i_colors_4;
in vec4 i_colors_5;
in vec4 i_colors_6;
in vec4 i_colors_7;
in vec4 i_colors_8;
in vec4 i_offsets_1;
in vec4 i_offsets_2;
in vec4 i_direction;

out vec2 v_raw_position;
out vec4 v_colors[8];
out vec4 v_offsets[2];
out vec4 v_direction;

void main() {
    gl_Position = u_transform * vec4(i_position, 0.0, 1.0);

    v_raw_position = i_position;

    v_colors[0] = i_colors_1;

    // array initializers are not supported in GLSL 1.0 (ES 2.0)
    v_colors[0] = i_colors_1;
    v_colors[1] = i_colors_2;
    v_colors[2] = i_colors_3;
    v_colors[3] = i_colors_4;
    v_colors[4] = i_colors_5;
    v_colors[5] = i_colors_6;
    v_colors[6] = i_colors_7;
    v_colors[7] = i_colors_8;

    v_offsets[0] = i_offsets_1;
    v_offsets[1] = i_offsets_2;

    v_direction = i_direction;
}
