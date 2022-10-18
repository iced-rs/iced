struct Uniforms {
    transform: mat4x4<f32>,
    //xy = start, wz = end
    position: vec4<f32>,
    //x = start stop, y = end stop, zw = padding
    stop_range: vec4<i32>,
}

struct Stop {
    color: vec4<f32>,
    offset: f32,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read> color_stops: array<Stop>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) raw_position: vec2<f32>
}

@vertex
fn vs_main(@location(0) input: vec2<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.transform * vec4<f32>(input.xy, 0.0, 1.0);
    output.raw_position = input;

    return output;
}

//TODO: rewrite without branching
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let start = uniforms.position.xy;
    let end = uniforms.position.zw;
    let start_stop = uniforms.stop_range.x;
    let end_stop = uniforms.stop_range.y;

    let v1 = end - start;
    let v2 = input.raw_position.xy - start;
    let unit = normalize(v1);
    let offset = dot(unit, v2) / length(v1);

    let min_stop = color_stops[start_stop];
    let max_stop = color_stops[end_stop];

    var color: vec4<f32>;

    if (offset <= min_stop.offset) {
        color = min_stop.color;
    } else if (offset >= max_stop.offset) {
        color = max_stop.color;
    } else {
        var min = min_stop;
        var max = max_stop;
        var min_index = start_stop;
        var max_index = end_stop;

        loop {
            if (min_index >= max_index - 1) {
                break;
            }

            let index = min_index + (max_index - min_index) / 2;

            let stop = color_stops[index];

            if (offset <= stop.offset) {
                max = stop;
                max_index = index;
            } else {
                min = stop;
                min_index = index;
            }
        }

        color = mix(min.color, max.color, smoothstep(
            min.offset,
            max.offset,
            offset
        ));
    }

    return color;
}
