struct Globals {
    transform: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> globals: Globals;

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) raw_position: vec2<f32>,
    @location(1) color_1: vec4<f32>,
    @location(2) color_2: vec4<f32>,
    @location(3) color_3: vec4<f32>,
    @location(4) color_4: vec4<f32>,
    @location(5) color_5: vec4<f32>,
    @location(6) color_6: vec4<f32>,
    @location(7) color_7: vec4<f32>,
    @location(8) color_8: vec4<f32>,
    @location(9) offsets_1: vec4<f32>,
    @location(10) offsets_2: vec4<f32>,
    @location(11) direction: vec4<f32>,
}

@vertex
fn vs_main(
    @location(0) input: vec2<f32>,
    @location(1) color_1: vec4<f32>,
    @location(2) color_2: vec4<f32>,
    @location(3) color_3: vec4<f32>,
    @location(4) color_4: vec4<f32>,
    @location(5) color_5: vec4<f32>,
    @location(6) color_6: vec4<f32>,
    @location(7) color_7: vec4<f32>,
    @location(8) color_8: vec4<f32>,
    @location(9) offsets_1: vec4<f32>,
    @location(10) offsets_2: vec4<f32>,
    @location(11) direction: vec4<f32>,
) -> GradientVertexOutput {
    var output: GradientVertexOutput;

    output.position = globals.transform * vec4<f32>(input.xy, 0.0, 1.0);
    output.raw_position = input;
    //pass gradient data to frag shader
    output.color_1 = color_1;
    output.color_2 = color_2;
    output.color_3 = color_3;
    output.color_4 = color_4;
    output.color_5 = color_5;
    output.color_6 = color_6;
    output.color_7 = color_7;
    output.color_8 = color_8;
    output.offsets_1 = offsets_1;
    output.offsets_2 = offsets_2;
    output.direction = direction;

    return output;
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

    return color;
}

@fragment
fn fs_main(input: GradientVertexOutput) -> @location(0) vec4<f32> {
    let colors = array<vec4<f32>, 8>(
        input.color_1,
        input.color_2,
        input.color_3,
        input.color_4,
        input.color_5,
        input.color_6,
        input.color_7,
        input.color_8,
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

    //TODO could just pass this in to the shader but is probably more performant to just check it here
    var last_index = 7;
    for (var i: i32 = 0; i <= 7; i++) {
        if (offsets[i] >= 1.0) {
            last_index = i;
            break;
        }
    }

    return gradient(input.raw_position, input.direction, colors, offsets, last_index);
}
