#version 450

layout(location = 0) uniform mat4 u_Transform;
layout(location = 1) uniform float u_Scale;

layout(location = 0) in vec2 i_Pos;
layout(location = 1) in vec2 i_Scale;
layout(location = 2) in vec4 i_Color;
layout(location = 3) in vec4 i_BorderColor;
layout(location = 4) in float i_BorderRadius;
layout(location = 5) in float i_BorderWidth;

layout(location = 0) out vec4 o_Color;
layout(location = 1) out vec4 o_BorderColor;
layout(location = 2) out vec2 o_Pos;
layout(location = 3) out vec2 o_Scale;
layout(location = 4) out float o_BorderRadius;
layout(location = 5) out float o_BorderWidth;

const vec2 positions[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    vec2 v_Pos = positions[gl_VertexID];
    vec2 p_Pos = i_Pos * u_Scale;
    vec2 p_Scale = i_Scale  * u_Scale;

    mat4 i_Transform = mat4(
        vec4(p_Scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4(0.0, p_Scale.y + 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(p_Pos - vec2(0.5, 0.5), 0.0, 1.0)
    );

    o_Color = i_Color;
    o_BorderColor = i_BorderColor;
    o_Pos = p_Pos;
    o_Scale = p_Scale;
    o_BorderRadius = i_BorderRadius * u_Scale;
    o_BorderWidth = i_BorderWidth * u_Scale;

    gl_Position = u_Transform * i_Transform * vec4(v_Pos, 0.0, 1.0);
}
