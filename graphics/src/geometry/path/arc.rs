//! Build and draw curves.
use iced_core::{Point, Vector};

/// A segment of a differentiable curve.
#[derive(Debug, Clone, Copy)]
pub struct Arc {
    /// The center of the arc.
    pub center: Point,
    /// The radius of the arc.
    pub radius: f32,
    /// The start of the segment's angle in radians, clockwise rotation from positive x-axis.
    pub start_angle: f32,
    /// The end of the segment's angle in radians, clockwise rotation from positive x-axis.
    pub end_angle: f32,
}

/// An elliptical [`Arc`].
#[derive(Debug, Clone, Copy)]
pub struct Elliptical {
    /// The center of the arc.
    pub center: Point,
    /// The radii of the arc's ellipse. The horizontal and vertical half-dimensions of the ellipse will match the x and y values of the radii vector.
    pub radii: Vector,
    /// The clockwise rotation of the arc's ellipse.
    pub rotation: f32,
    /// The start of the segment's angle in radians, clockwise rotation from positive x-axis.
    pub start_angle: f32,
    /// The end of the segment's angle in radians, clockwise rotation from positive x-axis.
    pub end_angle: f32,
}

impl From<Arc> for Elliptical {
    fn from(arc: Arc) -> Elliptical {
        Elliptical {
            center: arc.center,
            radii: Vector::new(arc.radius, arc.radius),
            rotation: 0.0,
            start_angle: arc.start_angle,
            end_angle: arc.end_angle,
        }
    }
}
