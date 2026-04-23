// Cached scale shader
// Renders an offscreen texture as a scaled/translated quad.
// Used for GPU-accelerated scale animations without re-rasterizing content.

struct Uniforms {
    // Source region in normalized texture coords (x, y, width, height)
    src_rect: vec4<f32>,
    // Destination quad in clip space: (x, y, width, height) in NDC
    dst_rect: vec4<f32>,
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
    // 6 vertices for two triangles forming a quad
    var local_uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    let local_uv = local_uvs[vertex_index];

    // Map local UV to destination clip-space position
    let x = u_uniforms.dst_rect.x + local_uv.x * u_uniforms.dst_rect.z;
    let y = u_uniforms.dst_rect.y + local_uv.y * u_uniforms.dst_rect.w;

    // Map local UV to source texture coordinates
    let tex_u = u_uniforms.src_rect.x + local_uv.x * u_uniforms.src_rect.z;
    let tex_v = u_uniforms.src_rect.y + local_uv.y * u_uniforms.src_rect.w;

    var output: VertexOutput;
    output.position = vec4<f32>(x, y, 0.0, 1.0);
    output.uv = vec2<f32>(tex_u, tex_v);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, input.uv);
}
