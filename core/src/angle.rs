use crate::{Point, Rectangle, Vector};
use std::f32::consts::PI;

#[derive(Debug, Copy, Clone, PartialEq)]
/// Degrees
pub struct Degrees(pub f32);

#[derive(Debug, Copy, Clone, PartialEq)]
/// Radians
pub struct Radians(pub f32);

impl From<Degrees> for Radians {
    fn from(degrees: Degrees) -> Self {
        Radians(degrees.0 * PI / 180.0)
    }
}

impl Radians {
    /// Calculates the line in which the [`Angle`] intercepts the `bounds`.
    pub fn to_distance(&self, bounds: &Rectangle) -> (Point, Point) {
        let v1 = Vector::new(f32::cos(self.0), f32::sin(self.0));

        let distance_to_rect = f32::min(
            f32::abs((bounds.y - bounds.center().y) / v1.y),
            f32::abs(((bounds.x + bounds.width) - bounds.center().x) / v1.x),
        );

        let start = bounds.center() + v1 * distance_to_rect;
        let end = bounds.center() - v1 * distance_to_rect;

        (start, end)
    }
}
