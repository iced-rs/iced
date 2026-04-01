struct SolidVertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) color: vec4<f32>,
    @location(1) pos: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_width: f32,
    @location(6) shadow_color: vec4<f32>,
    @location(7) shadow_offset: vec2<f32>,
    @location(8) shadow_blur_radius: f32,
    @location(9) shadow_inset: u32,
    @location(10) shadow_spread_radius: f32,
    @location(11) snap: u32,
}

struct SolidVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) border_color: vec4<f32>,
    @location(2) pos: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) border_radius: vec4<f32>,
    @location(5) border_width: f32,
    @location(6) shadow_color: vec4<f32>,
    @location(7) shadow_offset: vec2<f32>,
    @location(8) shadow_blur_radius: f32,
    @location(9) @interpolate(flat) shadow_inset: u32,
    @location(10) shadow_spread_radius: f32,
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var out: SolidVertexOutput;

    // For outset shadows, expand the quad bounds to include shadow area
    // For inset shadows, no expansion needed
    var shadow_expand = vec2<f32>(0.0, 0.0);
    if !bool(input.shadow_inset) {
        shadow_expand = min(input.shadow_offset, vec2<f32>(0.0, 0.0)) - input.shadow_blur_radius - max(input.shadow_spread_radius, 0.0);
    }

    var pos: vec2<f32> = (input.pos + shadow_expand) * globals.scale;
    var scale_expand = vec2<f32>(0.0, 0.0);
    if !bool(input.shadow_inset) {
        scale_expand = vec2<f32>(abs(input.shadow_offset.x), abs(input.shadow_offset.y)) + (input.shadow_blur_radius + max(input.shadow_spread_radius, 0.0)) * 2.0;
    }
    var scale: vec2<f32> = (input.scale + scale_expand) * globals.scale;

    var pos_snap = vec2<f32>(0.0, 0.0);
    var scale_snap = vec2<f32>(0.0, 0.0);

    if bool(input.snap) {
        pos_snap = round(pos + vec2(0.001, 0.001)) - pos;
        scale_snap = round(pos + scale + vec2(0.001, 0.001)) - pos - pos_snap - scale;
    }

    let border_radius = min(input.border_radius, vec4(min(input.scale.x, input.scale.y) / 2.0));

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(scale.x + scale_snap.x + 1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, scale.y + scale_snap.y + 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos + pos_snap - vec2<f32>(0.5, 0.5), 0.0, 1.0)
    );

    out.position = globals.transform * transform * vec4<f32>(vertex_position(input.vertex_index), 0.0, 1.0);
    out.color = premultiply(input.color);
    out.border_color = premultiply(input.border_color);
    out.pos = input.pos * globals.scale + pos_snap;
    out.scale = input.scale * globals.scale + scale_snap;
    out.border_radius = border_radius * globals.scale;
    out.border_width = input.border_width * globals.scale;
    out.shadow_color = premultiply(input.shadow_color);
    out.shadow_offset = input.shadow_offset * globals.scale;
    out.shadow_blur_radius = input.shadow_blur_radius * globals.scale;
    out.shadow_inset = input.shadow_inset;
    out.shadow_spread_radius = input.shadow_spread_radius * globals.scale;

    return out;
}

@fragment
fn solid_fs_main(
    input: SolidVertexOutput
) -> @location(0) vec4<f32> {
    var mixed_color: vec4<f32> = input.color;

    var dist = rounded_box_sdf(
        -(input.position.xy - input.pos - input.scale * 0.5) * 2.0,
        input.scale,
        input.border_radius * 2.0
    ) / 2.0;

    if (input.border_width > 0.0) {
        mixed_color = mix(
            input.color,
            input.border_color,
            clamp(0.5 + dist + input.border_width, 0.0, 1.0)
        );
    }

    var quad_alpha: f32 = clamp(0.5-dist, 0.0, 1.0);

    let quad_color = mixed_color * quad_alpha;

    if input.shadow_color.a > 0.0 {
        if bool(input.shadow_inset) {
            // Inset shadow - draw inside the quad
            // Spread contracts the inset shadow shape (positive spread = larger shadow area inside)
            let inset_spread = input.shadow_spread_radius;
            var inset_shadow_dist: f32 = rounded_box_sdf(
                -(input.position.xy - input.pos - input.shadow_offset - input.scale/2.0) * 2.0,
                input.scale - vec2(inset_spread * 2.0),
                max(input.border_radius * 2.0 - vec4(inset_spread * 2.0), vec4(0.0))
            ) / 2.0;
            // Invert the distance for inset effect
            let inset_alpha = 1.0 - smoothstep(-input.shadow_blur_radius, input.shadow_blur_radius, max(-inset_shadow_dist, 0.0));
            // Only apply shadow inside the quad (where quad_alpha > 0)
            return mix(quad_color, input.shadow_color * quad_alpha, inset_alpha * quad_alpha);
        } else {
            // Outset shadow - draw outside the quad
            // Spread expands the shadow shape (positive = larger shadow, negative = smaller)
            let spread = input.shadow_spread_radius;
            var shadow_dist: f32 = rounded_box_sdf(
                -(input.position.xy - input.pos - input.shadow_offset - input.scale/2.0) * 2.0,
                input.scale + vec2(spread * 2.0),
                max(input.border_radius * 2.0 + vec4(spread * 2.0), vec4(0.0))
            ) / 2.0;
            let shadow_alpha = 1.0 - smoothstep(-input.shadow_blur_radius, input.shadow_blur_radius, max(shadow_dist, 0.0));

            return mix(quad_color, input.shadow_color, (1.0 - quad_alpha) * shadow_alpha);
        }
    } else {
        return quad_color;
    }
}
