use crate::{Point, Rectangle, Vector};

use std::f32::consts::{FRAC_PI_2, PI};
use std::ops::RangeInclusive;

/// Degrees
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Degrees(pub f32);

/// Radians
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Radians(pub f32);

impl Radians {
    /// The range of radians of a circle.
    pub const RANGE: RangeInclusive<Radians> = Radians(0.0)..=Radians(2.0 * PI);
}

impl From<Degrees> for Radians {
    fn from(degrees: Degrees) -> Self {
        Self(degrees.0 * PI / 180.0)
    }
}

impl From<f32> for Radians {
    fn from(radians: f32) -> Self {
        Self(radians)
    }
}

impl From<u8> for Radians {
    fn from(radians: u8) -> Self {
        Self(f32::from(radians))
    }
}

impl From<Radians> for f64 {
    fn from(radians: Radians) -> Self {
        Self::from(radians.0)
    }
}

impl num_traits::FromPrimitive for Radians {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Self(n as f32))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(Self(n as f32))
    }

    fn from_f64(n: f64) -> Option<Self> {
        Some(Self(n as f32))
    }
}

impl Radians {
    /// Calculates the line in which the angle intercepts the `bounds`.
    pub fn to_distance(&self, bounds: &Rectangle) -> (Point, Point) {
        let angle = self.0 - FRAC_PI_2;
        let r = Vector::new(f32::cos(angle), f32::sin(angle));

        let distance_to_rect = f32::max(
            f32::abs(r.x * bounds.width / 2.0),
            f32::abs(r.y * bounds.height / 2.0),
        );

        let start = bounds.center() - r * distance_to_rect;
        let end = bounds.center() + r * distance_to_rect;

        (start, end)
    }
}
