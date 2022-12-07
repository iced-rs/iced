uniform mat4 u_Transform;
uniform float u_Scale;

attribute vec2 i_Pos;
attribute vec2 i_Scale;
attribute vec4 i_Color;
attribute vec4 i_BorderColor;
attribute vec4 i_BorderRadius;
attribute float i_BorderWidth;
attribute vec2 q_Pos;

varying vec4 v_Color;
varying vec4 v_BorderColor;
varying vec2 v_Pos;
varying vec2 v_Scale;
varying vec4 v_BorderRadius;
varying float v_BorderWidth;


void main() {
    vec2 p_Pos = i_Pos * u_Scale;
    vec2 p_Scale = i_Scale  * u_Scale;

    vec4 i_BorderRadius = vec4(
        min(i_BorderRadius.x, min(i_Scale.x, i_Scale.y) / 2.0),
        min(i_BorderRadius.y, min(i_Scale.x, i_Scale.y) / 2.0),
        min(i_BorderRadius.z, min(i_Scale.x, i_Scale.y) / 2.0),
        min(i_BorderRadius.w, min(i_Scale.x, i_Scale.y) / 2.0)
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
