struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
    // Rounded clip applied to the whole layer (physical px). `clip_bounds` is
    // [x, y, w, h]; `clip_radius` is per-corner. Ordinary layers pass a huge
    // rectangle with zero radius, so `layer_clip_alpha` stays 1.0 everywhere.
    clip_bounds: vec4<f32>,
    clip_radius: vec4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}

// Coverage (1 = keep, 0 = discard) of a fragment under the layer's rounded
// clip, with the same AA convention as the quad fill so trimmed corners stay
// smooth. `frag_pos` is the fragment's physical-pixel position.
fn layer_clip_alpha(frag_pos: vec2<f32>) -> f32 {
    let center = globals.clip_bounds.xy + globals.clip_bounds.zw * 0.5;
    let dist = rounded_box_sdf(
        -(frag_pos - center) * 2.0,
        globals.clip_bounds.zw,
        globals.clip_radius * 2.0
    ) / 2.0;
    return clamp(0.5 - dist, 0.0, 1.0);
}
