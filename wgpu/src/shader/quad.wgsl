struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn distance_alg(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    var inner_half_size: vec2<f32> = (size - vec2<f32>(radius, radius) * 2.0) / 2.0;
    var top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    return rounded_box_sdf(frag_coord - top_left - inner_half_size, inner_half_size, 0.0);
}

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
    var base_pos = (position + vertex_position(vertex_index) * input_scale) * global_scale;
    var ret: CornerPos;
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

fn rounded_box_sdf2(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}

// Given a vector from a point to the center of a rounded rectangle of the given `size` and
// border `radius`, determines the point's distance from the nearest edge of the rounded rectangle
fn rounded_box_sdf(to_center: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    return length(max(abs(to_center) - size + vec2<f32>(radius, radius), vec2<f32>(0.0, 0.0))) - radius;
}

// Based on the fragment position and the center of the quad, select one of the 4 radii.
// Order matches CSS border radius attribute:
// radii.x = top-left, radii.y = top-right, radii.z = bottom-right, radii.w = bottom-left
fn select_border_radius(radii: vec4<f32>, position: vec2<f32>, center: vec2<f32>) -> f32 {
    var rx = radii.x;
    var ry = radii.y;
    rx = select(radii.x, radii.y, position.x > center.x);
    ry = select(radii.w, radii.z, position.x > center.x);
    rx = select(rx, ry, position.y > center.y);
    return rx;
}
