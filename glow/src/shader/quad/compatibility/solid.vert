uniform mat4 u_transform;
uniform float u_scale;

attribute vec4 i_color;
attribute vec2 i_position;
attribute vec2 i_size;
attribute vec4 i_border_color;
attribute vec4 i_border_radius;
attribute float i_border_width;
attribute vec2 i_quad_position;

varying vec4 v_color;
varying vec2 v_position;
varying vec2 v_size;
varying vec4 v_border_color;
varying vec4 v_border_radius;
varying float v_border_width;

void main() {
    vec2 p_position = i_position * u_scale;
    vec2 p_size = i_size  * u_scale;

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

    v_color = i_color;
    v_border_color = i_border_color;
    v_position = p_position;
    v_size = p_size;
    v_border_radius = i_border_radius * u_scale;
    v_border_width = i_border_width * u_scale;

    gl_Position = u_transform * i_transform * vec4(i_quad_position, 0.0, 1.0);
}
