//! Build and draw curves.
use iced_native::{Point, Vector};

/// A segment of a differentiable curve.
#[derive(Debug, Clone, Copy)]
pub struct Arc {
    /// The center of the arc.
    pub center: Point,
    /// The radius of the arc.
    pub radius: f32,
    /// The start of the segment's angle, clockwise rotation.
    pub start_angle: f32,
    /// The end of the segment's angle, clockwise rotation.
    pub end_angle: f32,
}

/// An elliptical [`Arc`].
#[derive(Debug, Clone, Copy)]
pub struct Elliptical {
    /// The center of the arc.
    pub center: Point,
    /// The radii of the arc's ellipse, defining its axes.
    pub radii: Vector,
    /// The rotation of the arc's ellipse.
    pub rotation: f32,
    /// The start of the segment's angle, clockwise rotation.
    pub start_angle: f32,
    /// The end of the segment's angle, clockwise rotation.
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
