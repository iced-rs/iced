struct Uniforms {
    transform: mat4x4<f32>,
    color: vec4<f32>
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@location(0) input: vec2<f32>) -> @builtin(position) vec4<f32> {
    return uniforms.transform * vec4<f32>(input.xy, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return uniforms.color;
}
