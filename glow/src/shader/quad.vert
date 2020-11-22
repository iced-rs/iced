#version 330

uniform mat4 u_Transform;
uniform float u_Scale;

layout(location = 0) in vec2 i_Pos;
layout(location = 1) in vec2 i_Scale;
layout(location = 2) in vec4 i_Color;
layout(location = 3) in vec4 i_BorderColor;
layout(location = 4) in float i_BorderRadius;
layout(location = 5) in float i_BorderWidth;

out vec4 v_Color;
out vec4 v_BorderColor;
out vec2 v_Pos;
out vec2 v_Scale;
out float v_BorderRadius;
out float v_BorderWidth;

const vec2 positions[4] = vec2[](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    vec2 q_Pos = positions[gl_VertexID];
    vec2 p_Pos = i_Pos * u_Scale;
    vec2 p_Scale = i_Scale  * u_Scale;

    float i_BorderRadius = min(
        i_BorderRadius,
        min(i_Scale.x, i_Scale.y) / 2.0
    );

    mat4 i_Transform = mat4(
        vec4(p_Scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4(0.0, p_Scale.y + 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(p_Pos - vec2(0.5, 0.5), 0.0, 1.0)
    );

    v_Color = i_Color;
    v_BorderColor = i_BorderColor;
    v_Pos = p_Pos;
    v_Scale = p_Scale;
    v_BorderRadius = i_BorderRadius * u_Scale;
    v_BorderWidth = i_BorderWidth * u_Scale;

    gl_Position = u_Transform * i_Transform * vec4(q_Pos, 0.0, 1.0);
}
