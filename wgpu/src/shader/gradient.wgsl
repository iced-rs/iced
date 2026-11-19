@group(1) @binding(0) var gradient_stops: texture_2d<f32>;

fn gradient_texel(index: u32) -> vec4<f32> {
    let width = textureDimensions(gradient_stops).x;

    return textureLoad(gradient_stops, vec2<u32>(index % width, index / width), 0);
}

fn gradient_color(stops: vec2<u32>, i: u32) -> vec4<f32> {
    return gradient_texel(stops.x + i);
}

fn gradient_offset(stops: vec2<u32>, i: u32) -> f32 {
    return gradient_texel(stops.x + stops.y + i / 4u)[i % 4u];
}

fn random(coords: vec2<f32>) -> f32 {
    return fract(sin(dot(coords, vec2(12.9898,78.233))) * 43758.5453);
}

fn gradient(
    position: vec2<f32>,
    direction: vec4<f32>,
    stops: vec2<u32>,
) -> vec4<f32> {
    let count = stops.y;

    if (count == 0u) {
        return vec4<f32>(0.0);
    }

    let start = direction.xy;
    let end = direction.zw;

    let v1 = end - start;
    let v2 = position - start;
    let coord_offset = dot(v1, v2) / dot(v1, v1);

    let noise_granularity: f32 = 0.3/255.0;
    let noise = mix(-noise_granularity, noise_granularity, random(position));

    let last = count - 1u;

    var curr_offset = gradient_offset(stops, 0u);

    if (coord_offset <= curr_offset) {
        return gradient_color(stops, 0u) + noise;
    }

    if (coord_offset >= gradient_offset(stops, last)) {
        return gradient_color(stops, last) + noise;
    }

    for (var i: u32 = 0u; i < last; i++) {
        let next_offset = gradient_offset(stops, i + 1u);

        if (coord_offset <= next_offset) {
            let factor = smoothstep(curr_offset, next_offset, coord_offset);

            return interpolate_color(
                gradient_color(stops, i),
                gradient_color(stops, i + 1u),
                factor
            ) + noise;
        }

        curr_offset = next_offset;
    }

    return gradient_color(stops, last) + noise;
}
