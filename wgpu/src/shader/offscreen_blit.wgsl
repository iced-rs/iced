@group(0) @binding(0) var u_texture: texture_2d<f32>;
@group(0) @binding(1) var out_texture: texture_storage_2d<rgba8unorm, write>;

fn srgb(color: f32) -> f32 {
    if (color <= 0.0031308) {
        return 12.92 * color;
    } else {
        return (1.055 * (pow(color, (1.0/2.4)))) - 0.055;
    }
}

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    // texture coord must be i32 due to a naga bug:
    // https://github.com/gfx-rs/naga/issues/1997
    let coords = vec2(i32(id.x), i32(id.y));

    let src: vec4<f32> = textureLoad(u_texture, coords, 0);
    let srgb_color: vec4<f32> = vec4(srgb(src.x), srgb(src.y), srgb(src.z), src.w);

    textureStore(out_texture, coords, srgb_color);
}
