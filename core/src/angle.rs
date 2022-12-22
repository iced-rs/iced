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
