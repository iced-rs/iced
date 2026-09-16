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

/// Snaps `bounds` to the physical pixel grid, keeping signed coordinates.
///
/// It applies the same nudge-and-round convention as [`snap`], but keeps
/// floating-point coordinates, so negative values are preserved instead of
/// being saturated to zero.
///
/// Use this for coordinates that can legitimately extend beyond the
/// top-left corner of the viewport, like the bounds of content scrolled out
/// of view: those are clipped afterwards by their surroundings.
pub fn round(bounds: Rectangle) -> Rectangle {
    (bounds + Vector::new(NUDGE, NUDGE)).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Point, Size};

    /// An image scrolled out of view above the viewport must keep its
    /// negative position when snapped; saturating it to the origin would
    /// leave it stuck to the upper edge of the scrollable.
    #[test]
    fn round_keeps_negative_coordinates() {
        let bounds = Rectangle::new(Point::new(10.0, -50.0), Size::new(100.0, 100.0));

        assert_eq!(
            round(bounds),
            Rectangle::new(Point::new(10.0, -50.0), Size::new(100.0, 100.0))
        );
    }

    /// Coordinates landing a hair below a half-pixel boundary must round
    /// up onto the grid, like the `quad` vertex shader.
    #[test]
    fn round_applies_the_nudge() {
        let bounds = Rectangle::new(Point::new(0.0, 10.4998), Size::new(10.0, 1.0));

        // Without the nudge, plain `round` would land a pixel off:
        assert_eq!(bounds.y.round(), 10.0);

        // With it, the boundary rounds up, matching the GPU:
        assert_eq!(round(bounds).y, 11.0);
    }

    /// `snap` saturates negative coordinates to zero, so it must only be
    /// used on bounds that are known to be within the viewport, like
    /// scissor rects.
    #[test]
    fn snap_saturates_negative_coordinates() {
        let bounds = Rectangle::new(Point::new(10.0, -50.0), Size::new(100.0, 100.0));

        let snapped = snap(bounds).expect("Visible rectangle");

        assert_eq!(snapped.y, 0);
        assert_eq!(round(bounds).y, -50.0);
    }
}
