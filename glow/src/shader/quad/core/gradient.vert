uniform mat4 u_transform;
uniform float u_scale;

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
in vec4 i_position_and_size;
in vec4 i_border_color;
in vec4 i_border_radius;
in float i_border_width;

out vec4 v_colors[8];
out vec4 v_offsets[2];
out vec4 v_direction;
out vec4 v_position_and_size;
out vec4 v_border_color;
out vec4 v_border_radius;
out float v_border_width;

vec2 positions[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    vec2 i_position = i_position_and_size.xy;
    vec2 i_size = i_position_and_size.zw;

    vec2 q_position = positions[gl_VertexID];
    vec2 p_position = i_position * u_scale;
    vec2 p_size = i_size * u_scale;

    vec4 i_border_radius = vec4(
        min(i_border_radius.x, min(i_size.x, i_size.y) / 2.0),
        min(i_border_radius.y, min(i_size.x, i_size.y) / 2.0),
        min(i_border_radius.z, min(i_size.x, i_size.y) / 2.0),
        min(i_border_radius.w, min(i_size.x, i_size.y) / 2.0)
    );

    mat4 i_transform = mat4(
        vec4(p_size.x + 1.0, 0.0, 0.0, 0.0),
        vec4(0.0, p_size.y + 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(p_position - vec2(0.5, 0.5), 0.0, 1.0)
    );

    v_colors = vec4[](
        i_colors_1,
        i_colors_2,
        i_colors_3,
        i_colors_4,
        i_colors_5,
        i_colors_6,
        i_colors_7,
        i_colors_8
    );

    v_offsets = vec4[](
        i_offsets_1,
        i_offsets_2
    );

    v_direction = i_direction * u_scale;
    v_position_and_size = vec4(p_position, p_size);
    v_border_color = i_border_color;
    v_border_radius = i_border_radius * u_scale;
    v_border_width = i_border_width * u_scale;

    gl_Position = u_transform * i_transform * vec4(q_position, 0.0, 1.0);
}
