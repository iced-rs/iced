uniform mat4 u_transform;
uniform float u_scale;

//gradient
attribute vec4 i_colors_1;
attribute vec4 i_colors_2;
attribute vec4 i_colors_3;
attribute vec4 i_colors_4;
attribute vec4 i_colors_5;
attribute vec4 i_colors_6;
attribute vec4 i_colors_7;
attribute vec4 i_colors_8;
attribute vec4 i_offsets_1;
attribute vec4 i_offsets_2;
attribute vec4 i_direction;
//quad properties
attribute vec4 i_position_and_size;
attribute vec4 i_border_color;
attribute vec4 i_border_radius;
attribute float i_border_width;
attribute vec2 i_quad_position;

varying vec4 v_colors[8];
varying vec4 v_offsets[2];
varying vec4 v_direction;
varying vec4 v_position_and_size;
varying vec4 v_border_color;
varying vec4 v_border_radius;
varying float v_border_width;

void main() {
    vec2 i_position = i_position_and_size.xy;
    vec2 i_size = i_position_and_size.zw;

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

    v_direction = i_direction * u_scale;
    v_position_and_size = vec4(p_position, p_size);
    v_border_color = i_border_color;
    v_border_radius = i_border_radius * u_scale;
    v_border_width = i_border_width * u_scale;

    gl_Position = u_transform * i_transform * vec4(i_quad_position, 0.0, 1.0);
}
