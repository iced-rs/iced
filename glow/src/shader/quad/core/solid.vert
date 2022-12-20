uniform mat4 u_transform;
uniform float u_scale;

in vec4 i_color;
in vec2 i_position;
in vec2 i_size;
in vec4 i_border_color;
in vec4 i_border_radius;
in float i_border_width;

out vec4 v_color;
out vec4 v_border_color;
out vec2 v_position;
out vec2 v_scale;
out vec4 v_border_radius;
out float v_border_width;

vec2 positions[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    vec2 q_position = positions[gl_VertexID];
    vec2 p_position = i_position * u_scale;
    vec2 p_scale = i_size * u_scale;

    vec4 i_BorderRadius = vec4(
        min(i_border_radius.x, min(i_size.x, i_size.y) / 2.0),
        min(i_border_radius.y, min(i_size.x, i_size.y) / 2.0),
        min(i_border_radius.z, min(i_size.x, i_size.y) / 2.0),
        min(i_border_radius.w, min(i_size.x, i_size.y) / 2.0)
    );

    mat4 i_transform = mat4(
        vec4(p_scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4(0.0, p_scale.y + 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(p_position - vec2(0.5, 0.5), 0.0, 1.0)
    );

    v_color = i_color;
    v_border_color = i_border_color;
    v_position = p_position;
    v_scale = p_scale;
    v_border_radius = i_border_radius * u_scale;
    v_border_width = i_border_width * u_scale;

    gl_Position = u_transform * i_transform * vec4(q_position, 0.0, 1.0);
}
