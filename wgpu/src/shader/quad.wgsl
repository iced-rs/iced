struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn distance_alg(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    var inner_size: vec2<f32> = size - vec2<f32>(radius, radius) * 2.0;
    var top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    var bottom_right: vec2<f32> = top_left + inner_size;

    var top_left_distance: vec2<f32> = top_left - frag_coord;
    var bottom_right_distance: vec2<f32> = frag_coord - bottom_right;

    var dist: vec2<f32> = vec2<f32>(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0)
    );

    return sqrt(dist.x * dist.x + dist.y * dist.y);
}

// Based on the fragement position and the center of the quad, select one of the 4 radi.
// Order matches CSS border radius attribute:
// radi.x = top-left, radi.y = top-right, radi.z = bottom-right, radi.w = bottom-left
fn select_border_radius(radi: vec4<f32>, position: vec2<f32>, center: vec2<f32>) -> f32 {
    var rx = radi.x;
    var ry = radi.y;
    rx = select(radi.x, radi.y, position.x > center.x);
    ry = select(radi.w, radi.z, position.x > center.x);
    rx = select(rx, ry, position.y > center.y);
    return rx;
}

fn unpack_u32(color: u32) -> vec4<f32> {
    let u = unpack4x8unorm(color);

    return vec4<f32>(u.w, u.z, u.y, u.x);
}

struct SolidVertexInput {
    @location(0) v_pos: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) pos: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) border_color: vec4<f32>,
    @location(5) border_radius: vec4<f32>,
    @location(6) border_width: f32,
}

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) border_color: vec4<f32>,
    @location(2) pos: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_width: f32,
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var out: SolidVertexOutput;

    var pos: vec2<f32> = input.pos * globals.scale;
    var scale: vec2<f32> = input.scale * globals.scale;

    var min_border_radius = min(input.scale.x, input.scale.y) * 0.5;
    var border_radius: vec4<f32> = vec4<f32>(
        min(input.border_radius.x, min_border_radius),
        min(input.border_radius.y, min_border_radius),
        min(input.border_radius.z, min_border_radius),
        min(input.border_radius.w, min_border_radius)
    );

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y + 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos - vec2<f32>(0.5, 0.5), 0.0, 1.0)
    );

    out.position = globals.transform * transform * vec4<f32>(input.v_pos, 0.0, 1.0);
    out.color = input.color;
    out.border_color = input.border_color;
    out.pos = pos;
    out.scale = scale;
    out.border_radius = border_radius * globals.scale;
    out.border_width = input.border_width * globals.scale;

    return out;
}

@fragment
fn solid_fs_main(
    input: SolidVertexOutput
) -> @location(0) vec4<f32> {
    var mixed_color: vec4<f32> = input.color;

    var border_radius = select_border_radius(
        input.border_radius,
        input.position.xy,
        (input.pos + input.scale * 0.5).xy
    );

    if (input.border_width > 0.0) {
        var internal_border: f32 = max(border_radius - input.border_width, 0.0);

        var internal_distance: f32 = distance_alg(
            input.position.xy,
            input.pos + vec2<f32>(input.border_width, input.border_width),
            input.scale - vec2<f32>(input.border_width * 2.0, input.border_width * 2.0),
            internal_border
        );

        var border_mix: f32 = smoothstep(
            max(internal_border - 0.5, 0.0),
            internal_border + 0.5,
            internal_distance
        );

        mixed_color = mix(input.color, input.border_color, vec4<f32>(border_mix, border_mix, border_mix, border_mix));
    }

    var dist: f32 = distance_alg(
        vec2<f32>(input.position.x, input.position.y),
        input.pos,
        input.scale,
        border_radius
    );

    var radius_alpha: f32 = 1.0 - smoothstep(
        max(border_radius - 0.5, 0.0),
        border_radius + 0.5,
        dist
    );

    return vec4<f32>(mixed_color.x, mixed_color.y, mixed_color.z, mixed_color.w * radius_alpha);
}

struct GradientVertexInput {
    @location(0) v_pos: vec2<f32>,
    @location(1) colors_1: vec4<u32>,
    @location(2) colors_2: vec4<u32>,
    @location(3) offsets_1: vec4<f32>,
    @location(4) offsets_2: vec4<f32>,
    @location(5) direction: vec4<f32>,
    @location(6) position_and_scale: vec4<f32>,
    @location(7) border_color: vec4<f32>,
    @location(8) border_radius: vec4<f32>,
    @location(9) border_width: f32,
}

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) colors_1: vec4<u32>,
    @location(2) colors_2: vec4<u32>,
    @location(3) offsets_1: vec4<f32>,
    @location(4) offsets_2: vec4<f32>,
    @location(5) direction: vec4<f32>,
    @location(6) position_and_scale: vec4<f32>,
    @location(7) border_color: vec4<f32>,
    @location(8) border_radius: vec4<f32>,
    @location(9) border_width: f32,
}

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var out: GradientVertexOutput;

    var pos: vec2<f32> = input.position_and_scale.xy * globals.scale;
    var scale: vec2<f32> = input.position_and_scale.zw * globals.scale;

    var min_border_radius = min(input.position_and_scale.z, input.position_and_scale.w) * 0.5;
    var border_radius: vec4<f32> = vec4<f32>(
        min(input.border_radius.x, min_border_radius),
        min(input.border_radius.y, min_border_radius),
        min(input.border_radius.z, min_border_radius),
        min(input.border_radius.w, min_border_radius)
    );

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(scale.x + 1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y + 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos - vec2<f32>(0.5, 0.5), 0.0, 1.0)
    );

    out.position = globals.transform * transform * vec4<f32>(input.v_pos, 0.0, 1.0);
    out.colors_1 = input.colors_1;
    out.colors_2 = input.colors_2;
    out.offsets_1 = input.offsets_1;
    out.offsets_2 = input.offsets_2;
    out.direction = input.direction * globals.scale;
    out.position_and_scale = vec4<f32>(pos, scale);
    out.border_color = input.border_color;
    out.border_radius = border_radius * globals.scale;
    out.border_width = input.border_width * globals.scale;

    return out;
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
        unpack_u32(input.colors_1.x),
        unpack_u32(input.colors_1.y),
        unpack_u32(input.colors_1.z),
        unpack_u32(input.colors_1.w),
        unpack_u32(input.colors_2.x),
        unpack_u32(input.colors_2.y),
        unpack_u32(input.colors_2.z),
        unpack_u32(input.colors_2.w),
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
        if (offsets[i] > 1.0) {
            last_index = i - 1;
            break;
        }
    }

    var mixed_color: vec4<f32> = gradient(input.position.xy, input.direction, colors, offsets, last_index);

    let pos = input.position_and_scale.xy;
    let scale = input.position_and_scale.zw;

    var border_radius = select_border_radius(
        input.border_radius,
        input.position.xy,
        (pos + scale * 0.5).xy
    );

    if (input.border_width > 0.0) {
        var internal_border: f32 = max(border_radius - input.border_width, 0.0);

        var internal_distance: f32 = distance_alg(
            input.position.xy,
            pos + vec2<f32>(input.border_width, input.border_width),
            scale - vec2<f32>(input.border_width * 2.0, input.border_width * 2.0),
            internal_border
        );

        var border_mix: f32 = smoothstep(
            max(internal_border - 0.5, 0.0),
            internal_border + 0.5,
            internal_distance
        );

        mixed_color = mix(mixed_color, input.border_color, vec4<f32>(border_mix, border_mix, border_mix, border_mix));
    }

    var dist: f32 = distance_alg(
        input.position.xy,
        pos,
        scale,
        border_radius
    );

    var radius_alpha: f32 = 1.0 - smoothstep(
        max(border_radius - 0.5, 0.0),
        border_radius + 0.5,
        dist);

    return vec4<f32>(mixed_color.x, mixed_color.y, mixed_color.z, mixed_color.w * radius_alpha);
}
