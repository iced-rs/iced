#version 450

layout(location = 0) in vec2 v_Pos;
layout(location = 1) in vec2 i_Pos;
layout(location = 2) in vec2 i_Scale;
layout(location = 3) in vec4 i_Color;

layout (set = 0, binding = 0) uniform Globals {
    mat4 u_Transform;
};

layout(location = 0) out vec4 o_Color;

void main() {
    mat4 i_Transform = mat4(
        vec4(i_Scale.x, 0.0, 0.0, 0.0),
        vec4(0.0, i_Scale.y, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(i_Pos, 0.0, 1.0)
    );

    o_Color = i_Color;
    gl_Position = u_Transform * i_Transform * vec4(v_Pos, 0.0, 1.0);
}
