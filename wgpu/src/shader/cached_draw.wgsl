// Cached draw compositor shader
// Renders a cached texture at a given opacity within specified bounds

struct Uniforms {
    // Bounds in normalized coordinates (x, y, width, height)
    bounds: vec4<f32>,
    // params.x = opacity (0.0 to 1.0)
    // params.yzw = unused
    params: vec4<f32>,
}

@group(0) @binding(0) var u_sampler: sampler;
@group(0) @binding(1) var<uniform> u_uniforms: Uniforms;
@group(1) @binding(0) var u_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    let uv = uvs[vertex_index];

    // Map UV to the bounds region
    let x = u_uniforms.bounds.x + uv.x * u_uniforms.bounds.z;
    let y = u_uniforms.bounds.y + uv.y * u_uniforms.bounds.w;

    // Convert to clip space (-1 to 1)
    let clip_x = x * 2.0 - 1.0;
    let clip_y = 1.0 - y * 2.0;

    var output: VertexOutput;
    output.position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.uv = uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample from the bounds region of the texture
    let tex_coord = vec2<f32>(
        u_uniforms.bounds.x + input.uv.x * u_uniforms.bounds.z,
        u_uniforms.bounds.y + input.uv.y * u_uniforms.bounds.w
    );

    let color = textureSample(u_texture, u_sampler, tex_coord);
    let opacity = u_uniforms.params.x;

    // Premultiplied alpha: multiply all channels by opacity
    return color * opacity;
}
