struct Globals {
    transform: mat4x4<f32>,
    start: vec2<f32>,
    end: vec2<f32>,
    start_stop: i32,
    end_stop: i32,
};

struct Stop {
    offset: f32,
    color: vec4<f32>,
};

struct Stops {
    stops: array<Stop>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var<storage, read> color_stops: Stops;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = input.color;
    out.position = globals.transform * vec4<f32>(input.position, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let v1 = globals.end - globals.start;
    let v2 = input.position.xy - globals.start;
    let unit = normalize(v1);
    let offset = dot(unit, v2) / length(v1);

    let min_stop = color_stops.stops[globals.start_stop];
    let max_stop = color_stops.stops[globals.end_stop];

    var color: vec4<f32>;

    if (offset <= min_stop.offset) {
        color = min_stop.color;
    } else if (offset >= max_stop.offset) {
        color = max_stop.color;
    } else {
        var min = min_stop;
        var max = max_stop;
        var min_index = globals.start_stop;
        var max_index = globals.end_stop;

        loop {
            if (min_index >= max_index - 1) {
                break;
            }

            let index = min_index + (max_index - min_index) / 2;

            let stop = color_stops.stops[index];

            if (offset <= stop.offset) {
                max = stop;
                max_index = index;
            } else {
                min = stop;
                min_index = index;                
            }

        }

        let factor = (offset - min.offset) / (max.offset - min.offset);

        color = mix(min.color, max.color, factor);
    }

    return color;
}
