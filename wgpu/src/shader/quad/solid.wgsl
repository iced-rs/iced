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
}

struct CornerPos {
    pos: vec2<f32>,
    with_shadow: vec2<f32>,
    to_edge: vec2<f32>,
}
fn corner_pos(
    vertex_index: u32,
    input_scale: vec2<f32>,
    position: vec2<f32>,
    global_scale: f32,
    shadow_offset: vec2<f32>,
    shadow_blur_radius: f32,
) -> CornerPos {
    var base_pos = (position + vertex_position(vertex_index) * input_scale) * global_scale;
    var ret: CornerPos;
    switch vertex_index {
        case 0u, 5u: {
            ret.pos = vec2(ceil(base_pos.x), ceil(base_pos.y));
            ret.with_shadow = ret.pos
                + (vec2(
                    max(shadow_offset.x, 0.0),
                    max(shadow_offset.y, 0.0),
                ) + vec2(shadow_blur_radius, shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(0.5, 0.5);
        }
        case 1u: {
            ret.pos = vec2(ceil(base_pos.x), floor(base_pos.y));
            ret.with_shadow = ret.pos
                + (vec2(
                    max(shadow_offset.x, 0.0),
                    min(shadow_offset.y, 0.0),
                ) + vec2(shadow_blur_radius, -shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(0.5, -0.5);
        }
        case 2u, 3u: {
            ret.pos = vec2(floor(base_pos.x), floor(base_pos.y));
            ret.with_shadow = ret.pos
                + (vec2(
                    min(shadow_offset.x, 0.0),
                    min(shadow_offset.y, 0.0),
                ) + vec2(-shadow_blur_radius, -shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(-0.5, -0.5);
        }
        case 4u: {
            ret.pos = vec2(floor(base_pos.x), ceil(base_pos.y));
            ret.with_shadow = ret.pos
                + (vec2(
                    min(shadow_offset.x, 0.0),
                    max(shadow_offset.y, 0.0),
                ) + vec2(-shadow_blur_radius, shadow_blur_radius)) * global_scale;
            ret.to_edge = vec2(-0.5, 0.5);
        }
        default: {
            ret.pos = vec2<f32>();
            ret.with_shadow = vec2<f32>();
            ret.to_edge = vec2<f32>();
        }
    }
    return ret;
}

@vertex
fn solid_vs_main(input: SolidVertexInput) -> SolidVertexOutput {
    var out: SolidVertexOutput;

    var min_border_radius = min(input.scale.x, input.scale.y) * 0.5;
    var border_radius: vec4<f32> = vec4<f32>(
        min(input.border_radius.x, min_border_radius),
        min(input.border_radius.y, min_border_radius),
        min(input.border_radius.z, min_border_radius),
        min(input.border_radius.w, min_border_radius)
    );

    var cpos = corner_pos(
        input.vertex_index,
        input.scale,
        input.pos,
        globals.scale,
        input.shadow_offset,
        input.shadow_blur_radius,
    );
    // #: 0 1 2 3 4 5
    // x: 1 1 0 0 0 1
    // y: 1 0 0 0 1 1
    var other_index: u32;
    switch input.vertex_index {
        case 0u, 5u: {
            other_index = 2;
        }
        case 1u: {
            other_index = 4;
        }
        case 2u, 3u: {
            other_index = 0;
        }
        case 4u: {
            other_index = 1;
        }
        default: {
            other_index = 0;
        }
    }
    var other_cpos = corner_pos(
        other_index,
        input.scale,
        input.pos,
        globals.scale,
        input.shadow_offset,
        input.shadow_blur_radius,
    );

    out.position = globals.transform * vec4<f32>(cpos.with_shadow + cpos.to_edge, 0.0, 1.0);
    out.color = premultiply(input.color);
    out.border_color = premultiply(input.border_color);
    out.pos = vec2(min(cpos.pos.x, other_cpos.pos.x), min(cpos.pos.y, other_cpos.pos.y)); // input.pos * globals.scale - cpos.difference;
    out.scale = abs(cpos.pos - other_cpos.pos);
    out.border_radius = border_radius * globals.scale;
    out.border_width = input.border_width * globals.scale;
    out.shadow_color = premultiply(input.shadow_color);
    out.shadow_offset = input.shadow_offset * globals.scale;
    out.shadow_blur_radius = input.shadow_blur_radius * globals.scale;

    return out;
}

@fragment
fn solid_fs_main(
    input: SolidVertexOutput
) -> @location(0) vec4<f32> {
    var mixed_color: vec4<f32> = input.color;

    var dist: f32 = rounded_box_sdf2(
        -(input.position.xy - input.pos - input.scale/2.0) * 2.0,
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


    var radius_alpha: f32 = clamp(0.5-dist, 0.0, 1.0);

    let quad_color = mixed_color * radius_alpha;

    if input.shadow_color.a > 0.0 {
        var shadow_dist: f32 = rounded_box_sdf2(
            -(input.position.xy - input.pos - input.shadow_offset - input.scale/2.0) * 2.0,
            input.scale,
            input.border_radius * 2.0
        ) / 2.0;
        let shadow_alpha = 1.0 - smoothstep(-input.shadow_blur_radius, input.shadow_blur_radius, max(shadow_dist, 0.0));

        return mix(quad_color, input.shadow_color, (1.0 - radius_alpha) * shadow_alpha);
    } else {
        return quad_color;
    }
}
