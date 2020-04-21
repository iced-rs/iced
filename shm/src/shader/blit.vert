#version 450

layout(location = 0) out vec2 o_Uv;

const vec2 positions[6] = vec2[6](
    vec2(-1.0, 1.0),
    vec2(-1.0, -1.0),
    vec2(1.0, -1.0),
    vec2(-1.0, 1.0),
    vec2(1.0, 1.0),
    vec2(1.0, -1.0)
);

const vec2 uvs[6] = vec2[6](
    vec2(0.0, 0.0),
    vec2(0.0, 1.0),
    vec2(1.0, 1.0),
    vec2(0.0, 0.0),
    vec2(1.0, 0.0),
    vec2(1.0, 1.0)
);

void main() {
    o_Uv = uvs[gl_VertexIndex];
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
