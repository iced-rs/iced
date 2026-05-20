// WebGL2-compatible gradient shader with reduced inter-stage varyings
// Supports max 4 gradient stops instead of 8 to stay within WebGL2's limits
// Also supports radial gradients (gradient_type == 1)
// Includes shadow support by packing colors and flags to fit within 31 varyings

// Reduced input struct for WebGL2 - skips colors_3, colors_4, and _padding
// Uses explicit offsets in Rust to read from correct buffer positions
struct GradientVertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) @interpolate(flat) colors_1: vec4<u32>,
    @location(1) @interpolate(flat) colors_2: vec4<u32>,
    // Skip colors_3 (loc 2) and colors_4 (loc 3) - not used in WebGL 4-stop mode
    @location(2) @interpolate(flat) offsets: vec4<u32>,
    @location(3) direction: vec4<f32>,
    @location(4) gradient_type: u32,
    // Skip _padding - not needed
    @location(5) position_and_scale: vec4<f32>,
    @location(6) border_color: vec4<f32>,
    @location(7) border_radius: vec4<f32>,
    @location(8) border_widths: vec4<f32>,
    @location(9) shadow_color: vec4<f32>,
    @location(10) shadow_offset: vec2<f32>,
    @location(11) shadow_blur_radius: f32,
    // Packed: x = shadow_inset, y = snap, z = border_only, w = padding
    @location(12) flags: vec4<u32>,
}

// Reduced output struct for WebGL2 compatibility (max 31 inter-stage components)
// Pack border_color and shadow_color as u32 to save 6 components
// Pack gradient_type + shadow_inset + border_only into flags to save components
// Total: 4+4+4+4+4+1+4+1+1+2+1+1 = 31 components
struct GradientVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) colors_1: vec4<u32>,       // 4 components
    @location(1) @interpolate(flat) colors_2: vec4<u32>,       // 4 components
    @location(2) @interpolate(flat) offsets: vec4<f32>,        // 4 components
    @location(3) direction: vec4<f32>,                          // 4 components
    @location(4) position_and_scale: vec4<f32>,                 // 4 components
    @location(5) @interpolate(flat) border_color_packed: u32,  // 1 component (was 4)
    @location(6) border_radius: vec4<f32>,                      // 4 components
    @location(7) border_width: f32,                             // 1 component
    @location(8) @interpolate(flat) shadow_color_packed: u32,  // 1 component (was 4)
    @location(9) shadow_offset: vec2<f32>,                      // 2 components
    @location(10) shadow_blur_radius: f32,                      // 1 component
    @location(11) @interpolate(flat) flags: u32,               // 1 component (gradient_type + shadow_inset + border_only)
}

// Pack a vec4<f32> color (0.0-1.0) into a single u32 (RGBA8)
fn pack_color_to_u32(color: vec4<f32>) -> u32 {
    let r = u32(clamp(color.r, 0.0, 1.0) * 255.0);
    let g = u32(clamp(color.g, 0.0, 1.0) * 255.0);
    let b = u32(clamp(color.b, 0.0, 1.0) * 255.0);
    let a = u32(clamp(color.a, 0.0, 1.0) * 255.0);
    return (a << 24u) | (b << 16u) | (g << 8u) | r;
}

// Unpack a u32 (RGBA8) back to vec4<f32> color (0.0-1.0)
fn unpack_u32_to_color(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xFFu) / 255.0;
    let g = f32((packed >> 8u) & 0xFFu) / 255.0;
    let b = f32((packed >> 16u) & 0xFFu) / 255.0;
    let a = f32((packed >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

@vertex
fn gradient_vs_main(input: GradientVertexInput) -> GradientVertexOutput {
    var out: GradientVertexOutput;

    // Unpack input flags: x = shadow_inset, y = snap, z = border_only
    let shadow_inset = bool(input.flags.x);
    let snap = bool(input.flags.y);
    let border_only = input.flags.z;

    // For outset shadows, expand the quad bounds to include shadow area
    var shadow_expand = vec2<f32>(0.0, 0.0);
    if !shadow_inset {
        shadow_expand = min(input.shadow_offset, vec2<f32>(0.0, 0.0)) - input.shadow_blur_radius;
    }

    var pos: vec2<f32> = (input.position_and_scale.xy + shadow_expand) * globals.scale;
    var scale_expand = vec2<f32>(0.0, 0.0);
    if !shadow_inset {
        scale_expand = vec2<f32>(abs(input.shadow_offset.x), abs(input.shadow_offset.y)) + input.shadow_blur_radius * 2.0;
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
    
    // Unpack first 4 offsets to f32 in vertex shader
    let offsets_packed: vec4<f32> = unpack_u32(input.offsets.xy);
    out.offsets = offsets_packed;
    
    // For conic gradients (type 2), direction.z is angle in radians - don't scale it
    // For linear/radial, all components are coordinates that need scaling
    if (input.gradient_type == 2u) {
        out.direction = vec4<f32>(input.direction.xy * globals.scale, input.direction.z, input.direction.w);
    } else {
        out.direction = input.direction * globals.scale;
    }
    
    // Store original position/scale (without shadow expansion) for proper gradient sampling
    out.position_and_scale = vec4<f32>(input.position_and_scale.xy * globals.scale + pos_snap, input.position_and_scale.zw * globals.scale + scale_snap);
    
    // Pack border_color and shadow_color into u32 to save inter-stage components
    // Premultiply before packing
    out.border_color_packed = pack_color_to_u32(premultiply(input.border_color));
    out.border_radius = border_radius * globals.scale;
    // WebGL2: pass max border width as uniform (per-side not supported due to varying limit)
    out.border_width = max(max(input.border_widths.x, input.border_widths.y), max(input.border_widths.z, input.border_widths.w)) * globals.scale;
    out.shadow_color_packed = pack_color_to_u32(premultiply(input.shadow_color));
    out.shadow_offset = input.shadow_offset * globals.scale;
    out.shadow_blur_radius = input.shadow_blur_radius * globals.scale;
    
    // Pack gradient_type (bits 0-15) + shadow_inset (bit 16) + border_only (bit 17) into flags
    out.flags = input.gradient_type | (input.flags.x << 16u) | (border_only << 17u);

    return out;
}

fn random(coords: vec2<f32>) -> f32 {
    return fract(sin(dot(coords, vec2(12.9898,78.233))) * 43758.5453);
}

/// Returns the current interpolated color with a max 4-stop linear gradient (WebGL2 compatible)
fn gradient_linear(
    raw_position: vec2<f32>,
    direction: vec4<f32>,
    colors: array<vec4<f32>, 4>,
    offsets: array<f32, 4>,
    last_index: i32
) -> vec4<f32> {
    let start = direction.xy;
    let end = direction.zw;

    let v1 = end - start;
    let v2 = raw_position - start;
    let unit = normalize(v1);
    let coord_offset = dot(unit, v2) / length(v1);

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

/// Returns the current interpolated color with a max 4-stop radial gradient (WebGL2 compatible)
fn gradient_radial(
    raw_position: vec2<f32>,
    center: vec2<f32>,
    radius: vec2<f32>,
    colors: array<vec4<f32>, 4>,
    offsets: array<f32, 4>,
    last_index: i32
) -> vec4<f32> {
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

/// Returns the current interpolated color with a max 4-stop conic (angular) gradient (WebGL2 compatible)
/// For conic gradients, the gradient wraps around: after the last stop, it interpolates back to the first stop.
fn gradient_conic(
    raw_position: vec2<f32>,
    center: vec2<f32>,
    start_angle: f32,
    colors: array<vec4<f32>, 4>,
    offsets: array<f32, 4>,
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
    let colors = array<vec4<f32>, 4>(
        unpack_color(input.colors_1.xy),
        unpack_color(input.colors_1.zw),
        unpack_color(input.colors_2.xy),
        unpack_color(input.colors_2.zw),
    );

    var offsets = array<f32, 4>(
        input.offsets.x,
        input.offsets.y,
        input.offsets.z,
        input.offsets.w,
    );

    // Find last valid index (max 4 stops)
    var last_index = 3;
    for (var i: i32 = 0; i <= 3; i++) {
        if (offsets[i] > 1.0) {
            last_index = i - 1;
            break;
        }
    }

    // Unpack flags: low bits = gradient_type, bit 16 = shadow_inset, bit 17 = border_only
    let gradient_type = input.flags & 0xFFFFu;
    let shadow_inset = bool((input.flags >> 16u) & 1u);
    let border_only = bool((input.flags >> 17u) & 1u);
    
    // Unpack colors from u32
    let border_color = unpack_u32_to_color(input.border_color_packed);
    let shadow_color = unpack_u32_to_color(input.shadow_color_packed);

    var mixed_color: vec4<f32>;

    // Check gradient type: 0 = linear, 1 = radial, 2 = conic
    // direction contains: Linear: start.xy/end.zw, Radial: center.xy/radius.zw, Conic: center.xy/angle.z
    if (gradient_type == 2u) {
        mixed_color = gradient_conic(
            input.position.xy,
            input.direction.xy,  // center
            input.direction.z,   // start angle (radians)
            colors,
            offsets,
            last_index
        );
    } else if (gradient_type == 1u) {
        mixed_color = gradient_radial(
            input.position.xy,
            input.direction.xy,  // center (packed)
            input.direction.zw,  // radius (packed)
            colors,
            offsets,
            last_index
        );
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
    if (border_only && input.border_width > 0.0) {
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
            border_color,
            clamp(0.5 + dist + input.border_width, 0.0, 1.0)
        );
    }

    var quad_alpha: f32 = clamp(0.5-dist, 0.0, 1.0);

    let quad_color = mixed_color * quad_alpha;

    if shadow_color.a > 0.0 {
        if shadow_inset {
            // Inset shadow - draw inside the quad
            var inset_shadow_dist: f32 = rounded_box_sdf(
                -(input.position.xy - pos - input.shadow_offset - scale/2.0) * 2.0,
                scale,
                input.border_radius * 2.0
            ) / 2.0;
            // Invert the distance for inset effect
            let inset_alpha = 1.0 - smoothstep(-input.shadow_blur_radius, input.shadow_blur_radius, max(-inset_shadow_dist, 0.0));
            // Only apply shadow inside the quad (where quad_alpha > 0)
            return mix(quad_color, shadow_color * quad_alpha, inset_alpha * quad_alpha);
        } else {
            // Outset shadow - draw outside the quad
            var shadow_dist: f32 = rounded_box_sdf(
                -(input.position.xy - pos - input.shadow_offset - scale/2.0) * 2.0,
                scale,
                input.border_radius * 2.0
            ) / 2.0;
            let shadow_alpha = 1.0 - smoothstep(-input.shadow_blur_radius, input.shadow_blur_radius, max(shadow_dist, 0.0));

            return mix(quad_color, shadow_color, (1.0 - quad_alpha) * shadow_alpha);
        }
    } else {
        return quad_color;
    }
}
