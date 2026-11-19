struct GradientVertexInput {
    @location(0) v_pos: vec2<f32>,
}

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) raw_position: vec2<f32>,
}

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var output: GradientVertexOutput;

    output.position = globals.transform * vec4<f32>(input.v_pos, 0.0, 1.0);
    output.raw_position = input.v_pos;

    return output;
}

@fragment
fn gradient_fs_main(input: GradientVertexOutput) -> @location(0) vec4<f32> {
    return gradient(input.raw_position, globals.gradient_direction, globals.gradient_stops.xy);
}
