struct Globals {
    transform: mat4x4<f32>,
    scale: f32,
}

@group(0) @binding(0) var<uniform> globals: Globals;

fn rounded_box_sdf(p: vec2<f32>, size: vec2<f32>, corners: vec4<f32>) -> f32 {
    var box_half = select(corners.yz, corners.xw, p.x > 0.0);
    var corner = select(box_half.y, box_half.x, p.y > 0.0);
    var q = abs(p) - size + corner;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - corner;
}

// Returns the coverage of a border that is inset independently on every edge.
// Widths are ordered top, right, bottom, left.
fn border_coverage(
    position: vec2<f32>,
    size: vec2<f32>,
    radius: vec4<f32>,
    widths: vec4<f32>
) -> f32 {
    let inner_size = size - vec2<f32>(widths.y + widths.w, widths.x + widths.z);

    if inner_size.x <= 0.0 || inner_size.y <= 0.0 {
        return select(0.0, 1.0, any(widths > vec4<f32>(0.0)));
    }

    // Insetting a circle independently on the horizontal and vertical axes
    // produces an ellipse. Keep both radii instead of reducing each corner to
    // a circle using the largest adjacent border width.
    let inner_radius_x = max(
        radius - vec4<f32>(widths.w, widths.y, widths.y, widths.w),
        vec4<f32>(0.0)
    );
    let inner_radius_y = max(
        radius - vec4<f32>(widths.x, widths.x, widths.z, widths.z),
        vec4<f32>(0.0)
    );
    let distance = elliptical_rounded_box_sdf(
        position,
        vec2<f32>(widths.w, widths.x),
        size - vec2<f32>(widths.y, widths.z),
        inner_radius_x,
        inner_radius_y
    );

    return clamp(0.5 + distance, 0.0, 1.0);
}

// Returns an approximate signed distance to a rounded box whose corner radii
// may differ on the horizontal and vertical axes. The contour is exact, which
// is the important property for border coverage; the approximation is only
// used for the one-pixel antialiasing transition.
fn elliptical_rounded_box_sdf(
    position: vec2<f32>,
    inset_min: vec2<f32>,
    inset_max: vec2<f32>,
    radius_x: vec4<f32>,
    radius_y: vec4<f32>
) -> f32 {
    if (
        radius_x.x > 0.0 && radius_y.x > 0.0 &&
        position.x < inset_min.x + radius_x.x &&
        position.y < inset_min.y + radius_y.x
    ) {
        return elliptical_corner_sdf(
            position,
            inset_min + vec2<f32>(radius_x.x, radius_y.x),
            vec2<f32>(radius_x.x, radius_y.x)
        );
    }

    if (
        radius_x.y > 0.0 && radius_y.y > 0.0 &&
        position.x > inset_max.x - radius_x.y &&
        position.y < inset_min.y + radius_y.y
    ) {
        return elliptical_corner_sdf(
            position,
            vec2<f32>(inset_max.x - radius_x.y, inset_min.y + radius_y.y),
            vec2<f32>(radius_x.y, radius_y.y)
        );
    }

    if (
        radius_x.z > 0.0 && radius_y.z > 0.0 &&
        position.x > inset_max.x - radius_x.z &&
        position.y > inset_max.y - radius_y.z
    ) {
        return elliptical_corner_sdf(
            position,
            inset_max - vec2<f32>(radius_x.z, radius_y.z),
            vec2<f32>(radius_x.z, radius_y.z)
        );
    }

    if (
        radius_x.w > 0.0 && radius_y.w > 0.0 &&
        position.x < inset_min.x + radius_x.w &&
        position.y > inset_max.y - radius_y.w
    ) {
        return elliptical_corner_sdf(
            position,
            vec2<f32>(inset_min.x + radius_x.w, inset_max.y - radius_y.w),
            vec2<f32>(radius_x.w, radius_y.w)
        );
    }

    return max(
        max(inset_min.x - position.x, position.x - inset_max.x),
        max(inset_min.y - position.y, position.y - inset_max.y)
    );
}

fn elliptical_corner_sdf(
    position: vec2<f32>,
    center: vec2<f32>,
    radius: vec2<f32>
) -> f32 {
    return (length((position - center) / radius) - 1.0) * min(radius.x, radius.y);
}

// The smallest normalized distance selects a CSS-like diagonal join at a
// corner, while zero-width sides are never selected.
fn border_color_at(
    position: vec2<f32>,
    size: vec2<f32>,
    widths: vec4<f32>,
    top: vec4<f32>,
    right: vec4<f32>,
    bottom: vec4<f32>,
    left: vec4<f32>
) -> vec4<f32> {
    let disabled = 1e20;
    let distances = vec4<f32>(
        select(disabled, position.y / widths.x, widths.x > 0.0),
        select(disabled, (size.x - position.x) / widths.y, widths.y > 0.0),
        select(disabled, (size.y - position.y) / widths.z, widths.z > 0.0),
        select(disabled, position.x / widths.w, widths.w > 0.0)
    );
    let minimum = min(min(distances.x, distances.y), min(distances.z, distances.w));

    if minimum == distances.x { return top; }
    if minimum == distances.y { return right; }
    if minimum == distances.z { return bottom; }
    return left;
}
