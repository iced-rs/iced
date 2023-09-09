fn interpolate_color(from_: vec4<f32>, to_: vec4<f32>, factor: f32) -> vec4<f32> {
    return mix(from_, to_, factor);
}
