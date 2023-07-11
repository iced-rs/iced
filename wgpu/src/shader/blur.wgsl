struct Uniforms {
    texture_size: vec2<u32>,
    direction: f32,
    blur: f32,
}

var<private> positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0)
);

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(positions[index], 0.0, 1.0);
}

// Returns the gaussian weight of the offset
fn gaussian(offset: f32, sigma: f32) -> f32 {
    return 1.0 / (sqrt(2.0 * 3.141592) * sigma) * exp(-(offset * offset) / (2.0 * sigma * sigma));
}

fn offset(pos: vec2<f32>, i: f32) -> vec2<f32> {
    var offset: vec2<f32>;

    if (uniforms.direction >= 0.5) { //vertical
        offset = vec2<f32>(
            pos.x / f32(uniforms.texture_size.x),
            (pos.y - i) / f32(uniforms.texture_size.y),
        );
    } else { //horizontal
        offset = vec2<f32>(
            (pos.x - i) / f32(uniforms.texture_size.x),
            pos.y / f32(uniforms.texture_size.y),
        );
    }

    return offset;
}

@fragment
fn fs_main(@builtin(position) clip_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let sigma = uniforms.blur;
    let kernel_w = u32(2.0 * ceil( 3.0 * sigma ) + 1.0);

    var color = vec4<f32>(0.0);
    var total = 0.0;

    for (var i = 0u; i < kernel_w; i++) {
        let i_2 = f32(i) - (f32(kernel_w) / 2.0);
        let offset = offset(clip_pos.xy, i_2);
        let weight = gaussian(i_2, sigma);
        color += textureSample(texture, texture_sampler, offset) * weight;
        total += weight;
    }

    return color / total;
}
