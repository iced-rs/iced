struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn l(c: f32) -> f32 {
    if (c < 0.04045) {
        return c / 12.92;
    } else {
        return pow(((c + 0.055) / 1.055), 2.4);
    };
}

fn to_linear(color: u32) -> vec4<f32> {
    let c = unpack4x8unorm(color); //unpacks as a b g r
    return vec4<f32>(l(c.w), l(c.z), l(c.y), c.x);
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

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) raw_position: vec2<f32>,
    @location(1) colors_1: vec4<u32>,
    @location(2) colors_2: vec4<u32>,
    @location(3) offsets_1: vec4<f32>,
    @location(4) offsets_2: vec4<f32>,
    @location(5) direction: vec4<f32>,
}

@vertex
fn gradient_vs_main(
    @location(0) input: vec2<f32>,
    @location(1) colors_1: vec4<u32>,
    @location(2) colors_2: vec4<u32>,
    @location(3) offsets_1: vec4<f32>,
    @location(4) offsets_2: vec4<f32>,
    @location(5) direction: vec4<f32>,
) -> GradientVertexOutput {
    var output: GradientVertexOutput;

    output.position = globals.transform * vec4<f32>(input.xy, 0.0, 1.0);
    output.raw_position = input;
    output.colors_1 = colors_1;
    output.colors_2 = colors_2;
    output.offsets_1 = offsets_1;
    output.offsets_2 = offsets_2;
    output.direction = direction;

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
        to_linear(input.colors_1.x),
        to_linear(input.colors_1.y),
        to_linear(input.colors_1.z),
        to_linear(input.colors_1.w),
        to_linear(input.colors_2.x),
        to_linear(input.colors_2.y),
        to_linear(input.colors_2.z),
        to_linear(input.colors_2.w),
    );

    var offsets = array<f32, 8>(
        input.offsets_1.x,
        input.offsets_1.y,
        input.offsets_1.z,
        input.offsets_1.w,
        input.offsets_2.x,
        input.offsets_2.y,
        input.offsets_2.z,
        input.offsets_2.w,
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
