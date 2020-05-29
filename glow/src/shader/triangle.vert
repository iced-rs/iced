#version 330

uniform mat4 u_Transform;

layout(location = 0) in vec2 i_Position;
layout(location = 1) in vec4 i_Color;

out vec4 v_Color;

void main() {
    gl_Position = u_Transform * vec4(i_Position, 0.0, 1.0);
    v_Color = i_Color;
}
