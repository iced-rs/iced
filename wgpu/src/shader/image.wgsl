struct Globals {
    transform: mat4x4<f32>,
    scale_factor: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var u_sampler: sampler;
@group(1) @binding(0) var u_texture: texture_2d_array<f32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,
    @location(1) center: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) rotation: f32,
    @location(4) opacity: f32,
    @location(5) atlas_pos: vec2<f32>,
    @location(6) atlas_scale: vec2<f32>,
    @location(7) layer: i32,
    @location(8) snap: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) layer: f32, // this should be an i32, but naga currently reads that as requiring interpolation.
    @location(2) opacity: f32,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Generate a vertex position in the range [0, 1] from the vertex index.
    var v_pos = vertex_position(input.vertex_index);

    // Map the vertex position to the atlas texture.
    out.uv = vec2<f32>(v_pos * input.atlas_scale + input.atlas_pos);
    out.layer = f32(input.layer);
    out.opacity = input.opacity;

    // Calculate the vertex position and move the center to the origin
    v_pos = input.pos + v_pos * input.scale - input.center;

    // Apply the rotation around the center of the image
    let cos_rot = cos(input.rotation);
    let sin_rot = sin(input.rotation);
    let rotate = mat4x4<f32>(
        vec4<f32>(cos_rot, sin_rot, 0.0, 0.0),
        vec4<f32>(-sin_rot, cos_rot, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    // Calculate the final position of the vertex
    out.position = vec4(vec2(globals.scale_factor), 1.0, 1.0) * (vec4<f32>(input.center, 0.0, 0.0) + rotate * vec4<f32>(v_pos, 0.0, 1.0));

    if bool(input.snap) {
        out.position = round(out.position);
    }

    out.position = globals.transform * out.position;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample the texture at the given UV coordinate and layer.
    return textureSample(u_texture, u_sampler, input.uv, i32(input.layer)) * vec4<f32>(1.0, 1.0, 1.0, input.opacity);
}
