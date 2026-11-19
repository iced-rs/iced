struct Globals {
    transform: mat4x4<f32>,
    // Only used by gradient meshes
    gradient_direction: vec4<f32>,
    gradient_stops: vec4<u32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
