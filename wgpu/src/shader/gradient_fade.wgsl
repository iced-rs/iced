// Gradient fade shader
// Renders a texture with a gradient alpha mask applied

struct Uniforms {
    // Bounds in normalized coordinates (x, y, width, height)
    bounds: vec4<f32>,
    // params.x = direction (0=TopToBottom, 1=BottomToTop, 2=LeftToRight, 3=RightToLeft,
    //                       4=VerticalBoth, 5=HorizontalBoth)
    // params.y = fade_start (0.0 to 1.0) - for combined modes, this is the fade size from edges
    // params.z = fade_end (0.0 to 1.0)
    // params.w = overflow_margin (normalized, extra space around bounds for shadows)
    params: vec4<f32>,
}

@group(0) @binding(0) var u_sampler: sampler;
@group(0) @binding(1) var<uniform> u_uniforms: Uniforms;
@group(1) @binding(0) var u_texture: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Generate a fullscreen quad that covers the bounds area plus overflow margin
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Full screen UVs for the 6 vertices of two triangles
    var uvs = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0)
    );

    let uv = uvs[vertex_index];
    let margin = u_uniforms.params.w;

    // Expand the quad by the overflow margin on all sides
    let expanded_x = u_uniforms.bounds.x - margin;
    let expanded_y = u_uniforms.bounds.y - margin;
    let expanded_w = u_uniforms.bounds.z + margin * 2.0;
    let expanded_h = u_uniforms.bounds.w + margin * 2.0;

    // Map UV to the expanded bounds region
    let x = expanded_x + uv.x * expanded_w;
    let y = expanded_y + uv.y * expanded_h;
    
    // Convert to clip space (-1 to 1)
    let clip_x = x * 2.0 - 1.0;
    let clip_y = 1.0 - y * 2.0;

    var output: VertexOutput;
    output.position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.uv = uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let margin = u_uniforms.params.w;

    // Calculate expanded bounds for texture sampling
    let expanded_x = u_uniforms.bounds.x - margin;
    let expanded_y = u_uniforms.bounds.y - margin;
    let expanded_w = u_uniforms.bounds.z + margin * 2.0;
    let expanded_h = u_uniforms.bounds.w + margin * 2.0;

    // Calculate texture coordinates from the expanded UV
    let tex_coord = vec2<f32>(
        expanded_x + input.uv.x * expanded_w,
        expanded_y + input.uv.y * expanded_h
    );

    // Calculate position relative to original bounds (0..1) for fade
    let rel_x = (tex_coord.x - u_uniforms.bounds.x) / u_uniforms.bounds.z;
    let rel_y = (tex_coord.y - u_uniforms.bounds.y) / u_uniforms.bounds.w;
    
    let color = textureSample(u_texture, u_sampler, tex_coord);
    
    // Calculate the fade progress based on direction
    // Direction determines which axis and whether to invert the fade direction
    let direction = u32(u_uniforms.params.x);
    let fade_start = u_uniforms.params.y;
    let fade_end = u_uniforms.params.z;
    
    var alpha: f32;
    
    // Handle combined modes (4=VerticalBoth, 5=HorizontalBoth)
    if direction == 4u {
        // VerticalBoth: fade at both top and bottom edges
        // fade_end is the fade ratio (how far from each edge the fade extends)
        let fade_ratio = fade_end;
        
        // Top fade: transparent at 0, opaque at fade_ratio
        var top_alpha: f32;
        if rel_y <= 0.0 {
            top_alpha = 0.0;
        } else if rel_y >= fade_ratio {
            top_alpha = 1.0;
        } else {
            top_alpha = rel_y / fade_ratio;
        }
        
        // Bottom fade: opaque until (1 - fade_ratio), transparent at 1
        var bottom_alpha: f32;
        let bottom_start = 1.0 - fade_ratio;
        if rel_y <= bottom_start {
            bottom_alpha = 1.0;
        } else if rel_y >= 1.0 {
            bottom_alpha = 0.0;
        } else {
            bottom_alpha = 1.0 - (rel_y - bottom_start) / fade_ratio;
        }
        
        // Combine both fades (multiply)
        alpha = top_alpha * bottom_alpha;
    } else if direction == 5u {
        // HorizontalBoth: fade at both left and right edges
        // fade_end is the fade ratio (how far from each edge the fade extends)
        let fade_ratio = fade_end;
        
        // Left fade: transparent at 0, opaque at fade_ratio
        var left_alpha: f32;
        if rel_x <= 0.0 {
            left_alpha = 0.0;
        } else if rel_x >= fade_ratio {
            left_alpha = 1.0;
        } else {
            left_alpha = rel_x / fade_ratio;
        }
        
        // Right fade: opaque until (1 - fade_ratio), transparent at 1
        var right_alpha: f32;
        let right_start = 1.0 - fade_ratio;
        if rel_x <= right_start {
            right_alpha = 1.0;
        } else if rel_x >= 1.0 {
            right_alpha = 0.0;
        } else {
            right_alpha = 1.0 - (rel_x - right_start) / fade_ratio;
        }
        
        // Combine both fades (multiply)
        alpha = left_alpha * right_alpha;
    } else {
        // Single-edge modes (0-3)
        // Get the position along the fade axis (always 0-1 relative to original bounds)
        var position: f32;
        switch direction {
            // TopToBottom: vertical axis, top = 0, bottom = 1
            case 0u: {
                position = rel_y;
            }
            // BottomToTop: vertical axis, but fade direction is inverted
            case 1u: {
                position = rel_y;
            }
            // LeftToRight: horizontal axis, left = 0, right = 1
            case 2u: {
                position = rel_x;
            }
            // RightToLeft: horizontal axis, but fade direction is inverted
            case 3u: {
                position = rel_x;
            }
            default: {
                position = rel_y;
            }
        }
        
        // Calculate alpha based on position and fade range
        // For directions 0 and 2: fade OUT as we go from fade_start to fade_end
        // For directions 1 and 3: fade IN as we go from fade_start to fade_end
        if direction == 1u || direction == 3u {
            // Inverted fade (BottomToTop, RightToLeft): fade IN from start to end
            // Before fade_start: fully transparent
            // After fade_end: fully opaque
            if position <= fade_start {
                alpha = 0.0;
            } else if position >= fade_end {
                alpha = 1.0;
            } else {
                alpha = (position - fade_start) / (fade_end - fade_start);
            }
        } else {
            // Normal fade (TopToBottom, LeftToRight): fade OUT from start to end
            // Before fade_start: fully opaque
            // After fade_end: fully transparent
            if position <= fade_start {
                alpha = 1.0;
            } else if position >= fade_end {
                alpha = 0.0;
            } else {
                alpha = 1.0 - (position - fade_start) / (fade_end - fade_start);
            }
        }
    }
    
    // Apply alpha to the color (premultiplied alpha)
    return vec4<f32>(color.rgb * alpha, color.a * alpha);
}
