#version 450

layout(location = 0) uniform mat4 u_Transform;

layout(location = 0) in vec2 i_Position;
layout(location = 1) in vec4 i_Color;

layout(location = 0) out vec4 o_Color;

void main() {
    gl_Position = u_Transform * vec4(i_Position, 0.0, 1.0);
    o_Color = i_Color;
}
