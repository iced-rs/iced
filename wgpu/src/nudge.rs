//! Snapping coordinates to the physical pixel grid.
//!
//! # Why the nudge
//!
//! Snapping exists so that primitive edges land on the pixel grid and
//! render as crisp, fully opaque pixels instead of anti-aliased seams (see
//! [`CRISP`]). But `round`'s decision boundary — the half-pixel — is not
//! stable under floating-point arithmetic: a coordinate meant to land
//! exactly on a half-pixel (e.g. `5.25` logical pixels at 2× scale,
//! `10.5` physical) can be computed a hair below it (e.g. `10.4998`),
//! where plain `round` rounds down. The edge is then off the grid — the
//! anti-aliased seam that snapping exists to remove — and one pixel away
//! from its neighbours, which computed the same boundary cleanly.
//!
//! And half-pixels are common in UIs, not a corner case: centering and
//! equal space distribution are constant in layouts, and `.25` logical
//! pixels land exactly on physical half-pixels at 2× scale.
//!
//! Nudging each edge by [`NUDGE`] before rounding pulls the boundary down
//! to `.499`: any coordinate within `0.001` below a half-pixel now rounds
//! up onto the grid, so floating-point noise can no longer split a
//! half-pixel edge off the grid. Coordinates further from the boundary are
//! unaffected.
//!
//! # Consistency
//!
//! The same convention — nudging by [`NUDGE`] before rounding — is applied
//! by the `quad` vertex shader (see `shader/quad/snap.wgsl`) and by every
//! CPU-side snapping of these coordinates (the clip (scissor) bounds of the
//! quad, triangle, image and text passes). The two must never diverge, or
//! clip bounds may end up one pixel off from the snapped edges they clip.
//!
//! [`CRISP`]: crate::core::renderer::CRISP
use crate::core::{Rectangle, Vector};

/// The nudge applied when snapping a coordinate to the pixel grid.
///
/// Coordinates landing just below a half-pixel boundary round up because
/// of it.
///
/// This must stay in sync with `nudge` in
/// `shader/quad/snap.wgsl`.
pub const NUDGE: f32 = 0.001;

/// Snaps `bounds` to the physical pixel grid.
///
/// It uses the same rounding convention as the `quad` vertex shader
/// (see `shader/quad/snap.wgsl`): each edge is nudged by [`NUDGE`] before
/// being rounded, so that coordinates landing just below a half-pixel
/// boundary round up.
///
/// This ensures that clip (scissor) bounds align exactly with the snapped
/// edges of the [`Quad`] primitives drawn inside them.
///
/// Sub-pixel results are rejected, by [`Rectangle::snap`] itself.
///
/// [`Quad`]: crate::core::renderer::Quad
pub fn snap(bounds: Rectangle) -> Option<Rectangle<u32>> {
    (bounds + Vector::new(NUDGE, NUDGE)).snap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Point, Size};

    #[test]
    fn snap_matches_quad_shader_convention() {
        // Coordinates whose fractional part lands in `[0.499, 0.5)` must
        // round up, mirroring `round(edge + 0.001)` in the `quad` vertex
        // shader. Plain `round` would place these one pixel lower, making
        // clip bounds extend one pixel above snapped quad edges.
        let bounds = Rectangle {
            x: 10.4999,
            y: 10.4999,
            width: 100.0,
            height: 100.0,
        };

        assert_eq!(
            snap(bounds),
            Some(Rectangle {
                x: 11,
                y: 11,
                width: 100,
                height: 100
            })
        );

        // Values exactly on a half-pixel boundary already round up; the
        // nudge must not change them.
        let bounds = Rectangle {
            x: 10.5,
            y: 10.5,
            width: 100.0,
            height: 100.0,
        };

        assert_eq!(
            snap(bounds),
            Some(Rectangle {
                x: 11,
                y: 11,
                width: 100,
                height: 100
            })
        );

        // Sub-pixel bounds are rejected, like `Rectangle::snap`.
        assert_eq!(
            snap(Rectangle {
                x: 0.0,
                y: 0.0,
                width: 0.4,
                height: 100.0
            }),
            None
        );
    }

    /// Regression test for the thin-line glitch: a layer's clip bounds
    /// (scissor) must land on the same pixel row as the `Quad` edges snapped
    /// by the `quad` vertex shader.
    ///
    /// The old code snapped the scissor with plain `round`
    /// (`Rectangle::snap`) while the shader snapped with `round(edge + 0.001)`.
    /// Whenever a layer's physical Y coordinate had a fractional part in
    /// `[0.499, 0.5)`, the scissor started one physical pixel *above* the
    /// snapped top edge of the quads drawn inside it — so a `scrollable`'s
    /// clip could extend one pixel above its (and its siblings') snapped
    /// background, producing a 1px line in the final frame.
    #[test]
    fn layer_scissor_aligns_with_snapped_quad_edges() {
        for (layer_bounds, scale_factor) in [
            // 1x scale: physical Y = 10.4999, fractional part in the sliver
            (
                Rectangle::new(Point::new(0.0, 10.4999), Size::new(800.0, 400.0)),
                1.0,
            ),
            // 2x scale: logical Y = 5.2499, physical Y = 10.4998
            (
                Rectangle::new(Point::new(0.0, 5.2499), Size::new(800.0, 400.0)),
                2.0,
            ),
        ] {
            let viewport = Rectangle::with_size(Size::new(1920.0, 1080.0));
            let physical_bounds = layer_bounds * scale_factor;

            // As derived in `Renderer::draw`: the intersection of the
            // viewport's physical bounds with the layer's physical bounds,
            // snapped with [`snap`].
            let scissor = viewport
                .intersection(&physical_bounds)
                .and_then(snap)
                .expect("layer is fully visible");

            // What the `quad` vertex shader computes for the same top edge:
            // `round(pos + 0.001)` in physical pixels.
            let shader_snapped_y = (physical_bounds.y + NUDGE).round() as u32;

            assert_eq!(
                scissor.y, shader_snapped_y,
                "layer scissor must align with the shader-snapped quad edge \
                 (layer at {layer_bounds:?}, scale {scale_factor})"
            );

            // Document the failure mode: the old convention (plain `round`,
            // i.e. `Rectangle::snap`) produced a scissor one pixel *above*
            // the snapped quad edge for these bounds.
            let old_scissor_y = viewport
                .intersection(&physical_bounds)
                .and_then(Rectangle::snap)
                .expect("layer is fully visible")
                .y;

            assert_eq!(old_scissor_y + 1, shader_snapped_y);
        }
    }
}
