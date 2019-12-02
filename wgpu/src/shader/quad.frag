#version 450

layout(location = 0) in vec4 v_Color;
layout(location = 1) in vec4 v_BorderColor;
layout(location = 2) in vec2 v_Pos;
layout(location = 3) in vec2 v_Scale;
layout(location = 4) in float v_BorderRadius;
layout(location = 5) in float v_BorderWidth;

layout(location = 0) out vec4 o_Color;

float rounded(vec2 coord, vec2 pos, vec2 size, float radius){
    vec2 inner_size = size - vec2(2.0 * radius);
    vec2 top_left = pos + vec2(radius);
    vec2 bottom_right = top_left + inner_size;
    
    vec2 top_left_distance = top_left - coord;
    vec2 bottom_right_distance = coord - bottom_right;
    
    vec2 distance = vec2(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );
    
    float d = sqrt(distance.x * distance.x + distance.y * distance.y);
    return 1.0 - smoothstep(radius, radius + 0.5, d);
}

void main() {
    vec4 color = vec4(0);

    // border
    float border = min(v_BorderWidth, rounded(gl_FragCoord.xy, v_Pos, v_Scale, v_BorderRadius));
    color = mix(color, v_BorderColor, border);
    
    // content
    float content = rounded(
        gl_FragCoord.xy, 
        v_Pos + vec2(v_BorderWidth), 
        v_Scale - vec2(v_BorderWidth * 2.0), 
        max(v_BorderRadius - v_BorderWidth, 0.0)
    );
    color = mix(color, v_Color, content);
    
    o_Color = color;
}
