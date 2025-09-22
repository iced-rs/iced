fn premultiply(color: vec4<f32>) -> vec4<f32> {
    return vec4(color.xyz * color.a, color.a);
}

fn unpack_color(data: vec2<u32>) -> vec4<f32> {
    return premultiply(unpack_u32(data));
}

fn unpack_u32(data: vec2<u32>) -> vec4<f32> {
    let rg: vec2<f32> = unpack2x16float(data.x);
    let ba: vec2<f32> = unpack2x16float(data.y);

    return vec4<f32>(rg.y, rg.x, ba.y, ba.x);
}
