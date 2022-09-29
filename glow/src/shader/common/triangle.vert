uniform mat4 u_Transform;

in vec2 i_Position;
out vec2 raw_position;

void main() {
    gl_Position = u_Transform * vec4(i_Position, 0.0, 1.0);
    raw_position = i_Position;
}