// uniforms
struct SolidUniforms {
    transform: mat4x4<f32>,
    color: vec4<f32>
}

@group(0) @binding(0)
var<uniform> solid_uniforms: SolidUniforms;

@vertex
fn vs_main(@location(0) input: vec2<f32>) -> @builtin(position) vec4<f32> {
    return solid_uniforms.transform * vec4<f32>(input.xy, 0.0, 1.0);
}

@fragment
fn fs_solid() -> @location(0) vec4<f32> {
    return solid_uniforms.color;
}