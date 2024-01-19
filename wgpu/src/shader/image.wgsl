struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var u_sampler: sampler;
@group(1) @binding(0) var u_texture: texture_2d_array<f32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) scale: vec2<f32>,
    @location(2) atlas_pos: vec2<f32>,
    @location(3) atlas_scale: vec2<f32>,
    @location(4) layer: i32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: f32, // this should be an i32, but naga currently reads that as requiring interpolation.
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let v_pos = vertex_position(input.vertex_index);

    out.uv = vec2<f32>(v_pos * input.atlas_scale + input.atlas_pos);
    out.layer = f32(input.layer);

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(input.scale.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, input.scale.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(input.pos, 0.0, 1.0)
    );

    out.position = globals.transform * transform * vec4<f32>(v_pos, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, input.uv, i32(input.layer));
}
