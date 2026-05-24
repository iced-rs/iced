struct GradientVertexInput {
    @location(0) v_pos: vec2<f32>,
    @location(1) @interpolate(flat) colors_1: vec4<u32>,
    @location(2) @interpolate(flat) colors_2: vec4<u32>,
    @location(3) @interpolate(flat) colors_3: vec4<u32>,
    @location(4) @interpolate(flat) colors_4: vec4<u32>,
    @location(5) @interpolate(flat) offsets: vec4<u32>,
    @location(6) direction: vec4<f32>,
}

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) raw_position: vec2<f32>,
    @location(1) @interpolate(flat) colors_1: vec4<u32>,
    @location(2) @interpolate(flat) colors_2: vec4<u32>,
    @location(3) @interpolate(flat) colors_3: vec4<u32>,
    @location(4) @interpolate(flat) colors_4: vec4<u32>,
    @location(5) @interpolate(flat) offsets: vec4<u32>,
    @location(6) direction: vec4<f32>,
}

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var output: GradientVertexOutput;

    output.position = globals.transform * vec4<f32>(input.v_pos, 0.0, 1.0);
    output.raw_position = input.v_pos;
    output.colors_1 = input.colors_1;
    output.colors_2 = input.colors_2;
    output.colors_3 = input.colors_3;
    output.colors_4 = input.colors_4;
    output.offsets = input.offsets;
    output.direction = input.direction;

    return output;
}

fn select_color(
    c0: vec4<f32>, c1: vec4<f32>, c2: vec4<f32>, c3: vec4<f32>,
    c4: vec4<f32>, c5: vec4<f32>, c6: vec4<f32>, c7: vec4<f32>,
    i: i32
) -> vec4<f32> {
    if (i == 0) { return c0; }
    if (i == 1) { return c1; }
    if (i == 2) { return c2; }
    if (i == 3) { return c3; }
    if (i == 4) { return c4; }
    if (i == 5) { return c5; }
    if (i == 6) { return c6; }
    return c7;
}

fn select_offset(
    o0: f32, o1: f32, o2: f32, o3: f32,
    o4: f32, o5: f32, o6: f32, o7: f32,
    i: i32
) -> f32 {
    if (i == 0) { return o0; }
    if (i == 1) { return o1; }
    if (i == 2) { return o2; }
    if (i == 3) { return o3; }
    if (i == 4) { return o4; }
    if (i == 5) { return o5; }
    if (i == 6) { return o6; }
    return o7;
}

@fragment
fn gradient_fs_main(input: GradientVertexOutput) -> @location(0) vec4<f32> {
    let c0 = unpack_color(input.colors_1.xy);
    let c1 = unpack_color(input.colors_1.zw);
    let c2 = unpack_color(input.colors_2.xy);
    let c3 = unpack_color(input.colors_2.zw);
    let c4 = unpack_color(input.colors_3.xy);
    let c5 = unpack_color(input.colors_3.zw);
    let c6 = unpack_color(input.colors_4.xy);
    let c7 = unpack_color(input.colors_4.zw);

    let offsets_1 = unpack_u32(input.offsets.xy);
    let offsets_2 = unpack_u32(input.offsets.zw);

    let o0 = offsets_1.x;
    let o1 = offsets_1.y;
    let o2 = offsets_1.z;
    let o3 = offsets_1.w;
    let o4 = offsets_2.x;
    let o5 = offsets_2.y;
    let o6 = offsets_2.z;
    let o7 = offsets_2.w;

    var last_index = 7;
    if (o0 >= 1.0) { last_index = 0; }
    else if (o1 >= 1.0) { last_index = 1; }
    else if (o2 >= 1.0) { last_index = 2; }
    else if (o3 >= 1.0) { last_index = 3; }
    else if (o4 >= 1.0) { last_index = 4; }
    else if (o5 >= 1.0) { last_index = 5; }
    else if (o6 >= 1.0) { last_index = 6; }

    let start = input.direction.xy;
    let end = input.direction.zw;
    let v1 = end - start;
    let v2 = input.raw_position - start;
    let unit = normalize(v1);
    let coord_offset = dot(unit, v2) / length(v1);

    var color = vec4<f32>(0.0);
    let noise_granularity: f32 = 0.3 / 255.0;

    for (var i: i32 = 0; i < last_index; i++) {
        let curr_offset = select_offset(o0, o1, o2, o3, o4, o5, o6, o7, i);
        let next_offset = select_offset(o0, o1, o2, o3, o4, o5, o6, o7, i + 1);

        if (coord_offset <= o0) {
            color = c0;
        }

        if (curr_offset <= coord_offset && coord_offset <= next_offset) {
            let from_ = select_color(c0, c1, c2, c3, c4, c5, c6, c7, i);
            let to_   = select_color(c0, c1, c2, c3, c4, c5, c6, c7, i + 1);
            let factor = smoothstep(curr_offset, next_offset, coord_offset);
            color = mix(from_, to_, factor);
        }

        if (coord_offset >= select_offset(o0, o1, o2, o3, o4, o5, o6, o7, last_index)) {
            color = select_color(c0, c1, c2, c3, c4, c5, c6, c7, last_index);
        }
    }

    return color + mix(-noise_granularity, noise_granularity, random(input.raw_position));
}

fn random(coords: vec2<f32>) -> f32 {
    return fract(sin(dot(coords, vec2(12.9898, 78.233))) * 43758.5453);
}
