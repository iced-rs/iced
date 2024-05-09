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

// Given a vector from a point to the center of a rounded rectangle of the given `size` and
// border `radius`, determines the point's distance from the nearest edge of the rounded rectangle
fn rounded_box_sdf(to_center: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    return length(max(abs(to_center) - size + vec2<f32>(radius, radius), vec2<f32>(0.0, 0.0))) - radius;
}

// Based on the fragment position and the center of the quad, select one of the 4 radi.
// Order matches CSS border radius attribute:
// radi.x = top-left, radi.y = top-right, radi.z = bottom-right, radi.w = bottom-left
fn select_border_radius(radi: vec4<f32>, position: vec2<f32>, center: vec2<f32>) -> f32 {
    var rx = radi.x;
    var ry = radi.y;
    rx = select(radi.x, radi.y, position.x > center.x);
    ry = select(radi.w, radi.z, position.x > center.x);
    rx = select(rx, ry, position.y > center.y);
    return rx;
}
