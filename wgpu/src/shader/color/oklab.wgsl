const to_lms = mat3x4<f32>(
    vec4<f32>(0.4121656120,  0.2118591070,  0.0883097947, 0.0),
    vec4<f32>(0.5362752080,  0.6807189584,  0.2818474174, 0.0),
    vec4<f32>(0.0514575653,  0.1074065790,  0.6302613616, 0.0),
);

const to_rgb = mat3x4<f32>(
    vec4<f32>( 4.0767245293, -3.3072168827,  0.2307590544, 0.0),
    vec4<f32>(-1.2681437731,  2.6093323231, -0.3411344290, 0.0),
    vec4<f32>(-0.0041119885, -0.7034763098,  1.7068625689, 0.0),
);

fn interpolate_color(from_: vec4<f32>, to_: vec4<f32>, factor: f32) -> vec4<f32> {
    // To Oklab
    let lms_a = pow(from_ * to_lms, vec3<f32>(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0));
    let lms_b = pow(to_ * to_lms, vec3<f32>(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0));
    let mixed = mix(lms_a, lms_b, factor);

    // Back to linear RGB
    var color = to_rgb * (mixed * mixed * mixed);

    // Alpha interpolation
    color.a = mix(from_.a, to_.a, factor);

    return color;
}
