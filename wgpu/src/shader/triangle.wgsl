struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn unpack_u32(color: vec2<u32>) -> vec4<f32> {
    let rg: vec2<f32> = unpack2x16float(color.x);
    let ba: vec2<f32> = unpack2x16float(color.y);

    return vec4<f32>(rg.y, rg.x, ba.y, ba.x);
}

struct SolidVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var out: SolidVertexOutput;

    out.color = input.color;
    out.position = globals.transform * vec4<f32>(input.position, 0.0, 1.0);

    return out;
}

@fragment
fn solid_fs_main(input: SolidVertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}

struct GradientVertexInput {
    @location(0) v_pos: vec2<f32>,
    @location(1) colors_1: vec4<u32>,
    @location(2) colors_2: vec4<u32>,
    @location(3) colors_3: vec4<u32>,
    @location(4) colors_4: vec4<u32>,
    @location(5) offsets: vec4<u32>,
    @location(6) direction: vec4<f32>,
}

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) raw_position: vec2<f32>,
    @location(1) colors_1: vec4<u32>,
    @location(2) colors_2: vec4<u32>,
    @location(3) colors_3: vec4<u32>,
    @location(4) colors_4: vec4<u32>,
    @location(5) offsets: vec4<u32>,
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

fn random(coords: vec2<f32>) -> f32 {
    return fract(sin(dot(coords, vec2(12.9898,78.233))) * 43758.5453);
}

/// Returns the current interpolated color with a max 8-stop gradient
fn gradient(
    raw_position: vec2<f32>,
    direction: vec4<f32>,
    colors: array<vec4<f32>, 8>,
    offsets: array<f32, 8>,
    last_index: i32
) -> vec4<f32> {
    let start = direction.xy;
    let end = direction.zw;

    let v1 = end - start;
    let v2 = raw_position - start;
    let unit = normalize(v1);
    let coord_offset = dot(unit, v2) / length(v1);

    //need to store these as a var to use dynamic indexing in a loop
    //this is already added to wgsl spec but not in wgpu yet
    var colors_arr = colors;
    var offsets_arr = offsets;

    var color: vec4<f32>;

    let noise_granularity: f32 = 0.3/255.0;

    for (var i: i32 = 0; i < last_index; i++) {
        let curr_offset = offsets_arr[i];
        let next_offset = offsets_arr[i+1];

        if (coord_offset <= offsets_arr[0]) {
            color = colors_arr[0];
        }

        if (curr_offset <= coord_offset && coord_offset <= next_offset) {
            color = mix(colors_arr[i], colors_arr[i+1], smoothstep(
                curr_offset,
                next_offset,
                coord_offset,
            ));
        }

        if (coord_offset >= offsets_arr[last_index]) {
            color = colors_arr[last_index];
        }
    }

    return color + mix(-noise_granularity, noise_granularity, random(raw_position));
}

@fragment
fn gradient_fs_main(input: GradientVertexOutput) -> @location(0) vec4<f32> {
    let colors = array<vec4<f32>, 8>(
        unpack_u32(input.colors_1.xy),
        unpack_u32(input.colors_1.zw),
        unpack_u32(input.colors_2.xy),
        unpack_u32(input.colors_2.zw),
        unpack_u32(input.colors_3.xy),
        unpack_u32(input.colors_3.zw),
        unpack_u32(input.colors_4.xy),
        unpack_u32(input.colors_4.zw),
    );

    let offsets_1: vec4<f32> = unpack_u32(input.offsets.xy);
    let offsets_2: vec4<f32> = unpack_u32(input.offsets.zw);

    var offsets = array<f32, 8>(
        offsets_1.x,
        offsets_1.y,
        offsets_1.z,
        offsets_1.w,
        offsets_2.x,
        offsets_2.y,
        offsets_2.z,
        offsets_2.w,
    );

    var last_index = 7;
    for (var i: i32 = 0; i <= 7; i++) {
        if (offsets[i] >= 1.0) {
            last_index = i;
            break;
        }
    }

    return gradient(input.raw_position, input.direction, colors, offsets, last_index);
}
