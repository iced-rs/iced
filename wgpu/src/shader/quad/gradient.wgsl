struct GradientVertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) @interpolate(flat) colors_1: vec4<u32>,
    @location(1) @interpolate(flat) colors_2: vec4<u32>,
    @location(2) @interpolate(flat) colors_3: vec4<u32>,
    @location(3) @interpolate(flat) colors_4: vec4<u32>,
    @location(4) @interpolate(flat) offsets: vec4<u32>,
    @location(5) direction: vec4<f32>,  // Linear: start/end, Radial: center/radius
    @location(6) gradient_type: u32,
    @location(7) _padding: vec3<u32>,
    @location(8) position_and_scale: vec4<f32>,
    @location(9) border_color: vec4<f32>,
    @location(10) border_radius: vec4<f32>,
    @location(11) border_width: f32,
    @location(12) shadow_color: vec4<f32>,
    @location(13) shadow_offset: vec2<f32>,
    @location(14) shadow_blur_radius: f32,
    // Packed: x = shadow_inset, y = shadow_spread_radius (bitcast to f32), z = snap, w = border_only
    @location(15) flags: vec4<u32>,
}

struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) colors_1: vec4<u32>,
    @location(1) @interpolate(flat) colors_2: vec4<u32>,
    @location(2) @interpolate(flat) colors_3: vec4<u32>,
    @location(3) @interpolate(flat) colors_4: vec4<u32>,
    @location(4) @interpolate(flat) offsets: vec4<u32>,
    @location(5) direction: vec4<f32>,  // Linear: start/end, Radial: center/radius
    @location(6) @interpolate(flat) gradient_type: u32,
    @location(7) position_and_scale: vec4<f32>,
    @location(8) border_color: vec4<f32>,
    @location(9) border_radius: vec4<f32>,
    @location(10) border_width: f32,
    @location(11) shadow_color: vec4<f32>,
    @location(12) shadow_offset: vec2<f32>,
    // Packed: x = shadow_blur_radius, y = shadow_spread_radius
    @location(13) shadow_blur_and_spread: vec2<f32>,
    @location(14) @interpolate(flat) shadow_inset: u32,
    @location(15) @interpolate(flat) border_only: u32,
}

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var out: GradientVertexOutput;

    // Unpack flags: x = shadow_inset, y = shadow_spread_radius (bitcast), z = snap, w = border_only
    let shadow_inset = bool(input.flags.x);
    let shadow_spread_radius = bitcast<f32>(input.flags.y);
    let snap = bool(input.flags.z);

    // For outset shadows, expand the quad bounds to include shadow area
    // For inset shadows, no expansion needed
    var shadow_expand = vec2<f32>(0.0, 0.0);
    if !shadow_inset {
        shadow_expand = min(input.shadow_offset, vec2<f32>(0.0, 0.0)) - input.shadow_blur_radius - max(shadow_spread_radius, 0.0);
    }

    var pos: vec2<f32> = (input.position_and_scale.xy + shadow_expand) * globals.scale;
    var scale_expand = vec2<f32>(0.0, 0.0);
    if !shadow_inset {
        scale_expand = vec2<f32>(abs(input.shadow_offset.x), abs(input.shadow_offset.y)) + (input.shadow_blur_radius + max(shadow_spread_radius, 0.0)) * 2.0;
    }
    var scale: vec2<f32> = (input.position_and_scale.zw + scale_expand) * globals.scale;

    var pos_snap = vec2<f32>(0.0, 0.0);
    var scale_snap = vec2<f32>(0.0, 0.0);

    if snap {
        pos_snap = round(pos + vec2(0.001, 0.001)) - pos;
        scale_snap = round(pos + scale + vec2(0.001, 0.001)) - pos - pos_snap - scale;
    }

    var min_border_radius = min(input.position_and_scale.z, input.position_and_scale.w) * 0.5;
    var border_radius: vec4<f32> = vec4<f32>(
        min(input.border_radius.x, min_border_radius),
        min(input.border_radius.y, min_border_radius),
        min(input.border_radius.z, min_border_radius),
        min(input.border_radius.w, min_border_radius)
    );

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(scale.x + scale_snap.x + 1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y + scale_snap.y + 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos + pos_snap - vec2<f32>(0.5, 0.5), 0.0, 1.0)
    );

    out.position = globals.transform * transform * vec4<f32>(vertex_position(input.vertex_index), 0.0, 1.0);
    out.colors_1 = input.colors_1;
    out.colors_2 = input.colors_2;
    out.colors_3 = input.colors_3;
    out.colors_4 = input.colors_4;
    out.offsets = input.offsets;
    // For conic gradients (type 2), direction.z is angle in radians - don't scale it
    // For linear/radial, all components are coordinates that need scaling
    if (input.gradient_type == 2u) {
        out.direction = vec4<f32>(input.direction.xy * globals.scale, input.direction.z, input.direction.w);
    } else {
        out.direction = input.direction * globals.scale;
    }
    out.gradient_type = input.gradient_type;
    // Store original position/scale (without shadow expansion) for proper gradient sampling
    out.position_and_scale = vec4<f32>(input.position_and_scale.xy * globals.scale + pos_snap, input.position_and_scale.zw * globals.scale + scale_snap);
    out.border_color = premultiply(input.border_color);
    out.border_radius = border_radius * globals.scale;
    out.border_width = input.border_width * globals.scale;
    out.shadow_color = premultiply(input.shadow_color);
    out.shadow_offset = input.shadow_offset * globals.scale;
    out.shadow_blur_and_spread = vec2<f32>(input.shadow_blur_radius * globals.scale, shadow_spread_radius * globals.scale);
    out.shadow_inset = input.flags.x;
    out.border_only = input.flags.w;

    return out;
}

fn random(coords: vec2<f32>) -> f32 {
    return fract(sin(dot(coords, vec2(12.9898,78.233))) * 43758.5453);
}

/// Returns the current interpolated color with a max 8-stop gradient (linear)
fn gradient_linear(
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
            let from_ = colors_arr[i];
            let to_ = colors_arr[i+1];
            let factor = smoothstep(curr_offset, next_offset, coord_offset);

            color = interpolate_color(from_, to_, factor);
        }

        if (coord_offset >= offsets_arr[last_index]) {
            color = colors_arr[last_index];
        }
    }

    return color + mix(-noise_granularity, noise_granularity, random(raw_position));
}

/// Returns the current interpolated color with a max 8-stop radial gradient
fn gradient_radial(
    raw_position: vec2<f32>,
    center: vec2<f32>,
    radius: vec2<f32>,
    colors: array<vec4<f32>, 8>,
    offsets: array<f32, 8>,
    last_index: i32
) -> vec4<f32> {
    // Calculate normalized distance from center (elliptical)
    let diff = raw_position - center;
    let normalized_dist = length(diff / radius);

    var colors_arr = colors;
    var offsets_arr = offsets;

    var color: vec4<f32>;

    let noise_granularity: f32 = 0.3/255.0;

    for (var i: i32 = 0; i < last_index; i++) {
        let curr_offset = offsets_arr[i];
        let next_offset = offsets_arr[i+1];

        if (normalized_dist <= offsets_arr[0]) {
            color = colors_arr[0];
        }

        if (curr_offset <= normalized_dist && normalized_dist <= next_offset) {
            let from_ = colors_arr[i];
            let to_ = colors_arr[i+1];
            let factor = smoothstep(curr_offset, next_offset, normalized_dist);

            color = interpolate_color(from_, to_, factor);
        }

        if (normalized_dist >= offsets_arr[last_index]) {
            color = colors_arr[last_index];
        }
    }

    return color + mix(-noise_granularity, noise_granularity, random(raw_position));
}

/// Returns the current interpolated color with a max 8-stop conic (angular) gradient
/// For conic gradients, the gradient wraps around: after the last stop, it interpolates back to the first stop.
fn gradient_conic(
    raw_position: vec2<f32>,
    center: vec2<f32>,
    start_angle: f32,
    colors: array<vec4<f32>, 8>,
    offsets: array<f32, 8>,
    last_index: i32
) -> vec4<f32> {
    let PI: f32 = 3.141592653589793;
    let TWO_PI: f32 = 6.283185307179586;
    
    // Calculate vector from center to current position
    let diff = raw_position - center;
    
    // Compute angle using atan2
    // Note: In screen coordinates, Y increases downward, so we negate Y
    // to get standard mathematical angles (counterclockwise from right)
    let angle = atan2(-diff.y, diff.x);
    
    // Normalize to [0, 1] range, accounting for start angle
    // Add TWO_PI before fract to ensure positive result
    let normalized = (angle - start_angle + TWO_PI) / TWO_PI;
    let coord_offset = fract(normalized);

    var colors_arr = colors;
    var offsets_arr = offsets;

    var color: vec4<f32>;

    let noise_granularity: f32 = 0.3/255.0;

    // Handle wrap-around: if past the last stop, interpolate back to first stop
    if (coord_offset >= offsets_arr[last_index]) {
        let from_ = colors_arr[last_index];
        let to_ = colors_arr[0];
        // Calculate factor for wrap-around segment (from last_offset to 1.0, then 0.0 to first_offset)
        let wrap_length = (1.0 - offsets_arr[last_index]) + offsets_arr[0];
        let dist_from_last = coord_offset - offsets_arr[last_index];
        let factor = dist_from_last / wrap_length;
        color = interpolate_color(from_, to_, factor);
    } else if (coord_offset <= offsets_arr[0]) {
        // Before first stop - also part of wrap-around from last stop
        let from_ = colors_arr[last_index];
        let to_ = colors_arr[0];
        let wrap_length = (1.0 - offsets_arr[last_index]) + offsets_arr[0];
        let dist_from_last = (1.0 - offsets_arr[last_index]) + coord_offset;
        let factor = dist_from_last / wrap_length;
        color = interpolate_color(from_, to_, factor);
    } else {
        // Normal case: between two consecutive stops
        for (var i: i32 = 0; i < last_index; i++) {
            let curr_offset = offsets_arr[i];
            let next_offset = offsets_arr[i+1];

            if (curr_offset <= coord_offset && coord_offset <= next_offset) {
                let from_ = colors_arr[i];
                let to_ = colors_arr[i+1];
                let factor = smoothstep(curr_offset, next_offset, coord_offset);
                color = interpolate_color(from_, to_, factor);
                break;
            }
        }
    }

    return color + mix(-noise_granularity, noise_granularity, random(raw_position));
}

@fragment
fn gradient_fs_main(input: GradientVertexOutput) -> @location(0) vec4<f32> {
    let colors = array<vec4<f32>, 8>(
        unpack_color(input.colors_1.xy),
        unpack_color(input.colors_1.zw),
        unpack_color(input.colors_2.xy),
        unpack_color(input.colors_2.zw),
        unpack_color(input.colors_3.xy),
        unpack_color(input.colors_3.zw),
        unpack_color(input.colors_4.xy),
        unpack_color(input.colors_4.zw),
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

    //TODO could just pass this in to the shader but is probably more performant to just check it here
    var last_index = 7;
    for (var i: i32 = 0; i <= 7; i++) {
        if (offsets[i] > 1.0) {
            last_index = i - 1;
            break;
        }
    }

    var mixed_color: vec4<f32>;

    // Check gradient type: 0 = linear, 1 = radial, 2 = conic
    // direction contains: Linear: start.xy/end.zw, Radial: center.xy/radius.zw, Conic: center.xy/angle.z
    if (input.gradient_type == 2u) {
        mixed_color = gradient_conic(input.position.xy, input.direction.xy, input.direction.z, colors, offsets, last_index);
    } else if (input.gradient_type == 1u) {
        mixed_color = gradient_radial(input.position.xy, input.direction.xy, input.direction.zw, colors, offsets, last_index);
    } else {
        mixed_color = gradient_linear(input.position.xy, input.direction, colors, offsets, last_index);
    }

    let pos = input.position_and_scale.xy;
    let scale = input.position_and_scale.zw;

    var dist: f32 = rounded_box_sdf(
        -(input.position.xy - pos - scale / 2.0) * 2.0,
        scale,
        input.border_radius * 2.0
    ) / 2.0;

    // Handle border_only mode: gradient fills only the border region
    if (bool(input.border_only) && input.border_width > 0.0) {
        // dist is negative inside the quad, positive outside
        // Calculate inner boundary distance (where interior starts)
        let inner_dist = dist + input.border_width;
        
        // outer_alpha: 1.0 inside quad edge, 0.0 outside  
        // This fades in as we cross the outer boundary (dist goes from positive to negative)
        let outer_alpha = clamp(0.5 - dist, 0.0, 1.0);
        
        // inner_alpha: 0.0 in border region, 1.0 in interior
        // When inner_dist < 0 (in border or outside), we want this to be 0
        // When inner_dist > 0 (in interior), we want this to be 1
        let inner_alpha = clamp(0.5 - inner_dist, 0.0, 1.0);
        
        // Border region is inside the outer edge but outside the inner edge
        // outer_alpha = 1, inner_alpha = 0 → border_alpha = 1
        // outer_alpha = 1, inner_alpha = 1 → border_alpha = 0 (interior - hidden)
        let border_alpha = outer_alpha * (1.0 - inner_alpha);
        
        return mixed_color * border_alpha;
    }

    if (input.border_width > 0.0) {
        mixed_color = mix(
            mixed_color,
            input.border_color,
            clamp(0.5 + dist + input.border_width, 0.0, 1.0)
        );
    }

    var quad_alpha: f32 = clamp(0.5-dist, 0.0, 1.0);

    let quad_color = mixed_color * quad_alpha;

    if input.shadow_color.a > 0.0 {
        let blur = input.shadow_blur_and_spread.x;
        let spread = input.shadow_blur_and_spread.y;

        if bool(input.shadow_inset) {
            // Inset shadow - draw inside the quad
            // Spread contracts the inset shadow shape (positive spread = larger shadow area inside)
            var inset_shadow_dist: f32 = rounded_box_sdf(
                -(input.position.xy - pos - input.shadow_offset - scale/2.0) * 2.0,
                scale - vec2(spread * 2.0),
                max(input.border_radius * 2.0 - vec4(spread * 2.0), vec4(0.0))
            ) / 2.0;
            // Invert the distance for inset effect
            let inset_alpha = 1.0 - smoothstep(-blur, blur, max(-inset_shadow_dist, 0.0));
            // Only apply shadow inside the quad (where quad_alpha > 0)
            return mix(quad_color, input.shadow_color * quad_alpha, inset_alpha * quad_alpha);
        } else {
            // Outset shadow - draw outside the quad
            // Spread expands the shadow shape (positive = larger shadow, negative = smaller)
            var shadow_dist: f32 = rounded_box_sdf(
                -(input.position.xy - pos - input.shadow_offset - scale/2.0) * 2.0,
                scale + vec2(spread * 2.0),
                max(input.border_radius * 2.0 + vec4(spread * 2.0), vec4(0.0))
            ) / 2.0;
            let shadow_alpha = 1.0 - smoothstep(-blur, blur, max(shadow_dist, 0.0));

            return mix(quad_color, input.shadow_color, (1.0 - quad_alpha) * shadow_alpha);
        }
    } else {
        return quad_color;
    }
}
