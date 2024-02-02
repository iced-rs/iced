use crate::{Point, Rectangle, Vector};

use std::f32::consts::{FRAC_PI_2, PI};
use std::ops::{Add, AddAssign, Div, Mul, RangeInclusive, Sub, SubAssign};

/// Degrees
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Degrees(pub f32);

/// Radians
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Radians(pub f32);

impl Radians {
    /// The range of radians of a circle.
    pub const RANGE: RangeInclusive<Self> = Self(0.0)..=Self(2.0 * PI);

    /// The amount of radians in half a circle.
    pub const PI: Self = Self(PI);

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

impl Sub for Radians {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Radians {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

impl Add for Radians {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Radians {
    fn add_assign(&mut self, rhs: Radians) {
        self.0 = self.0 + rhs.0;
    }
}

impl Mul for Radians {
    type Output = Self;

    fn mul(self, rhs: Radians) -> Self::Output {
        Radians(self.0 * rhs.0)
    }
}

impl Mul<f32> for Radians {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<Radians> for f32 {
    type Output = Radians;

    fn mul(self, rhs: Radians) -> Self::Output {
        Radians(self * rhs.0)
    }
}

impl Div<f32> for Radians {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Radians(self.0 / rhs)
    }
}

impl Div for Radians {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}
