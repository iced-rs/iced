var<private> positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0)
);

var<private> uvs: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0)
);

@group(0) @binding(0) var depth_sampler: sampler;
@group(0) @binding(1) var depth_texture: texture_2d<f32>;

struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) v_index: u32) -> Output {
    var out: Output;

    out.position = vec4<f32>(positions[v_index], 0.0, 1.0);
    out.uv = uvs[v_index];

    return out;
}

@fragment
fn fs_main(input: Output) -> @location(0) vec4<f32> {
    let depth = textureSample(depth_texture, depth_sampler, input.uv).r;

    if (depth > .9999) {
        discard;
    }

    let c = 1.0 - depth;

    return vec4<f32>(c, c, c, 1.0);
}
