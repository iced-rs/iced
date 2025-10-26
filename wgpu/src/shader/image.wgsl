struct Globals {
    transform: mat4x4<f32>,
    scale_factor: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var u_sampler: sampler;
@group(1) @binding(0) var u_texture: texture_2d_array<f32>;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) center: vec2<f32>,
    @location(1) clip_bounds: vec4<f32>,
    @location(2) border_radius: vec4<f32>,
    @location(3) tile: vec4<f32>,
    @location(4) rotation: f32,
    @location(5) opacity: f32,
    @location(6) atlas_pos: vec2<f32>,
    @location(7) atlas_scale: vec2<f32>,
    @location(8) layer: i32,
    @location(9) snap: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) clip_bounds: vec4<f32>,
    @location(1) border_radius: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) layer: f32, // this should be an i32, but naga currently reads that as requiring interpolation.
    @location(4) opacity: f32,
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

    let tile = input.tile;
    let center = input.center;

    // Calculate the vertex position and move the center to the origin
    v_pos = tile.xy + v_pos * tile.zw - center;

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
    out.position = vec4(vec2(globals.scale_factor), 1.0, 1.0) * (vec4<f32>(center, 0.0, 0.0) + rotate * vec4<f32>(v_pos, 0.0, 1.0));

    if bool(input.snap) {
        out.position = round(out.position);
    }

    out.position = globals.transform * out.position;
    out.clip_bounds = globals.scale_factor * input.clip_bounds;
    out.border_radius = globals.scale_factor * input.border_radius;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let fragment = input.position.xy;
    let position = input.clip_bounds.xy;
    let scale = input.clip_bounds.zw;

    let d = rounded_box_sdf(
        2.0 * (fragment - position - scale / 2.0),
        scale,
        input.border_radius * 2.0,
    ) / 2.0;

    let antialias: f32 = clamp(1.0 - d, 0.0, 1.0);

    return textureSample(u_texture, u_sampler, input.uv, i32(input.layer)) * vec4<f32>(1.0, 1.0, 1.0, antialias * input.opacity);
}

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}
