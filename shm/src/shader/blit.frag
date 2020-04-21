#version 450

layout(location = 0) in vec2 v_Uv;

layout(set = 0, binding = 0) uniform sampler u_Sampler;
layout(set = 1, binding = 0) uniform texture2D u_Texture;

layout(location = 0) out vec4 o_Color;

void main() {
    o_Color = texture(sampler2D(u_Texture, u_Sampler), v_Uv);
}
