#version 450

layout(location = 0) in vec2 v_Color;
layout(location = 1) in vec2 v_Pos;
layout(location = 2) in vec2 v_Scale;
layout(location = 3) in float v_BorderRadius;

layout(set = 0, binding = 1) uniform sampler u_Sampler;
layout(set = 1, binding = 0) uniform texture2D u_Texture;

layout(location = 0) out vec4 o_Color;

float rounded(in vec2 frag_coord, in vec2 position, in vec2 size, float radius, float s)
{
    vec2 inner_size = size - vec2(radius, radius) * 2.0;
    vec2 top_left = position + vec2(radius, radius);
    vec2 bottom_right = top_left + inner_size;

    vec2 top_left_distance = top_left - frag_coord;
    vec2 bottom_right_distance = frag_coord - bottom_right;

    vec2 distance = vec2(
        max(max(top_left_distance.x, bottom_right_distance.x), 0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0)
    );

    float d = sqrt(distance.x * distance.x + distance.y * distance.y);

    return 1.0 - smoothstep(radius - s, radius + s, d);
}

vec4 gradient(in vec2 frag_coord, in vec2 position, in vec2 size, in vec2 color_data, in texture2D t, in sampler s)
{
    vec2 texture_size = textureSize(sampler2D(t, s) , 0);
    vec2 st = (frag_coord.xy - position) / size;
    float mix_value = distance(st.y, 0);

    float color_offset = ((color_data.x + 0.5) + (color_data.y - 1) * mix_value) / texture_size.y;

    vec2 color_pos = vec2(0.5, color_offset);
    return texture(sampler2D(t, s), color_pos);
}

void main() {
    float radius_alpha = rounded(gl_FragCoord.xy, v_Pos, v_Scale, v_BorderRadius, 0.5);
    vec4 color = gradient(gl_FragCoord.xy, v_Pos, v_Scale, v_Color, u_Texture, u_Sampler);
    o_Color = vec4(color.xyz, color.w * radius_alpha);

}
