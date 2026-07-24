var<private> positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0)
);

@group(0) @binding(0) var u_sampler: sampler;
// x = opacity, yzw = padding for 16-byte alignment.
@group(0) @binding(1) var<uniform> u_opacity: vec4<f32>;
@group(1) @binding(0) var u_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let p = positions[vertex_index];

    var out: VertexOutput;
    out.uv = p;
    out.position = vec4<f32>(p * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // The group texture stores premultiplied-alpha color. Scaling every channel
    // by the opacity keeps it premultiplied, so blending the result with a
    // premultiplied "over" produces correct group opacity: the group is
    // flattened first and faded as a whole.
    return textureSample(u_texture, u_sampler, input.uv) * u_opacity.x;
}
