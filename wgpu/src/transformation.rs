use nalgebra::Matrix3;
use std::ops::Mul;

/// A 2D transformation matrix.
///
/// It can be used to apply a transformation to a [`Target`].
///
/// [`Target`]: struct.Target.html
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation(Matrix3<f32>);

impl Transformation {
    /// Get the identity transformation.
    pub fn identity() -> Transformation {
        Transformation(Matrix3::identity())
    }

    /// Creates an orthographic projection.
    ///
    /// You should rarely need this. On creation, a [`Target`] is automatically
    /// set up with the correct orthographic projection.
    ///
    /// [`Target`]: struct.Target.html
    #[rustfmt::skip]
    pub fn orthographic(width: u16, height: u16) -> Transformation {
        Transformation(nalgebra::Matrix3::new(
            2.0 / f32::from(width), 0.0, -1.0,
            0.0, 2.0 / f32::from(height), -1.0,
            0.0, 0.0, 1.0
        ))
    }

    /// Creates a translate transformation.
    ///
    /// You can use this to pan your camera, for example.
    pub fn translate(x: f32, y: f32) -> Transformation {
        Transformation(Matrix3::new_translation(&nalgebra::Vector2::new(x, y)))
    }
}

impl Mul for Transformation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Transformation(self.0 * rhs.0)
    }
}

impl From<Transformation> for [f32; 16] {
    #[rustfmt::skip]
    fn from(t: Transformation) -> [f32; 16] {
        [
            t.0[0], t.0[1], 0.0, t.0[2],
            t.0[3], t.0[4], 0.0, t.0[5],
            0.0, 0.0, -1.0, 0.0,
            t.0[6], t.0[7], 0.0, t.0[8]
        ]
    }
}
