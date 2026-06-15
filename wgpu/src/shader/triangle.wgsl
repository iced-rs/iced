struct Globals {
    transform: mat4x4<f32>,
    clip_bounds: vec4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn discard_if_clipped(position: vec4<f32>) {
    let pixel = position.xy;

    if (
        pixel.x < globals.clip_bounds.x ||
        pixel.y < globals.clip_bounds.y ||
        pixel.x >= globals.clip_bounds.z ||
        pixel.y >= globals.clip_bounds.w
    ) {
        discard;
    }
}
