struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

struct CornerPos {
    pos: vec2<f32>,
    with_shadow: vec2<f32>,
    to_edge: vec2<f32>,
}

fn corner_pos(
    vertex_index: u32,
    input_scale: vec2<f32>,
    position: vec2<f32>,
    global_scale: f32,
    shadow_offset: vec2<f32>,
    shadow_blur_radius: f32,
) -> CornerPos {
    var ret: CornerPos;
    var base_pos = (position + vertex_position(vertex_index) * input_scale) * global_scale;
    ret.pos = round(base_pos);
    switch vertex_index {
        case 0u, 5u: {
            ret.with_shadow = ret.pos
                + (vec2(
                    max(shadow_offset.x, 0.0),
                    max(shadow_offset.y, 0.0),
                ) + vec2(shadow_blur_radius, shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(0.5, 0.5);
        }
        case 1u: {
            ret.with_shadow = ret.pos
                + (vec2(
                    max(shadow_offset.x, 0.0),
                    min(shadow_offset.y, 0.0),
                ) + vec2(shadow_blur_radius, -shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(0.5, -0.5);
        }
        case 2u, 3u: {
            ret.with_shadow = ret.pos
                + (vec2(
                    min(shadow_offset.x, 0.0),
                    min(shadow_offset.y, 0.0),
                ) + vec2(-shadow_blur_radius, -shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(-0.5, -0.5);
        }
        case 4u: {
            ret.with_shadow = ret.pos
                + (vec2(
                    min(shadow_offset.x, 0.0),
                    max(shadow_offset.y, 0.0),
                ) + vec2(-shadow_blur_radius, shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(-0.5, 0.5);
        }
        default: {
            ret.pos = vec2<f32>();
            ret.with_shadow = vec2<f32>();
            ret.to_edge = vec2<f32>();
        }
    }
    return ret;
}

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}
