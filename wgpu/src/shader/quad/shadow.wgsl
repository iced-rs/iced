/// Returns the bounds of the quad at `pos`/`scale` expanded to fully
/// contain its shadow (offset + blur).
///
/// The returned `vec4` is `[pos.x, pos.y, scale.x, scale.y]`.
fn shadow_expanded_bounds(
    pos: vec2<f32>,
    scale: vec2<f32>,
    shadow_offset: vec2<f32>,
    shadow_blur_radius: f32
) -> vec4<f32> {
    return vec4<f32>(
        pos + min(shadow_offset, vec2<f32>(0.0, 0.0)) - shadow_blur_radius,
        scale + vec2<f32>(abs(shadow_offset.x), abs(shadow_offset.y)) + shadow_blur_radius * 2.0
    );
}

/// Mixes the shadow of the quad with bounds `pos`/`scale` (the quad
/// itself, not the shadow-expanded bounds) into `quad_color`.
///
/// The shadow is only blended in where the quad itself is transparent
/// (`1.0 - quad_alpha`), so it never covers the quad's own color.
fn mix_shadow(
    quad_color: vec4<f32>,
    quad_alpha: f32,
    position: vec2<f32>,
    pos: vec2<f32>,
    scale: vec2<f32>,
    border_radius: vec4<f32>,
    shadow_color: vec4<f32>,
    shadow_offset: vec2<f32>,
    shadow_blur_radius: f32
) -> vec4<f32> {
    var dist: f32 = rounded_box_sdf(
        -(position - pos - shadow_offset - scale / 2.0) * 2.0,
        scale,
        border_radius * 2.0
    ) / 2.0;

    let shadow_alpha = 1.0 - smoothstep(-shadow_blur_radius, shadow_blur_radius, max(dist, 0.0));

    return mix(quad_color, shadow_color, (1.0 - quad_alpha) * shadow_alpha);
}
