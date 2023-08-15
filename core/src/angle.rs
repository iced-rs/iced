use crate::{Point, Rectangle, Vector};
use std::{f32::consts::PI, ops::{SubAssign, AddAssign, Add, Sub}};

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

impl SubAssign<Radians> for Radians {
    fn sub_assign(&mut self, rhs: Radians) {
        self.0 = self.0 - rhs.0;
    }
}

impl AddAssign<Radians> for Radians {
    fn add_assign(&mut self, rhs: Radians) {
        self.0 = self.0 + rhs.0;
    }
}

impl Add<Radians> for Radians {
    type Output = Radians;

    fn add(self, rhs: Radians) -> Self::Output {
        Radians(self.0 + rhs.0)
    }
}

impl Sub<Radians> for Radians {
    type Output = Radians;

    fn sub(self, rhs: Radians) -> Self::Output {
        Radians(self.0 - rhs.0)
    }
}