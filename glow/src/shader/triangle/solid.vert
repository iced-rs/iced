uniform mat4 u_transform;

in vec2 i_position;
in vec4 i_color;

out vec4 v_color;

void main() {
    gl_Position = u_transform * vec4(i_position, 0.0, 1.0);
    v_color = i_color;
}
