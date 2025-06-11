use crate::{Point, Rectangle, Size, Vector};

use glam::{Mat4, Vec3, Vec4};
use std::ops::Mul;

/// A 2D transformation matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transformation(Mat4);

impl Transformation {
    /// A [`Transformation`] that preserves whatever is transformed.
    pub const IDENTITY: Self = Self(Mat4::IDENTITY);

    /// Creates an orthographic projection.
    #[rustfmt::skip]
    pub fn orthographic(width: u32, height: u32) -> Self{
        Self(Mat4::orthographic_rh_gl(
            0.0, width as f32,
            height as f32, 0.0,
            -1.0, 1.0
        ))
    }

    /// Creates a translate transformation.
    pub fn translate(x: f32, y: f32) -> Self {
        Self(Mat4::from_translation(Vec3::new(x, y, 0.0)))
    }

    /// Creates a uniform scaling transformation.
    pub fn scale(scaling: f32) -> Self {
        Self(Mat4::from_scale(Vec3::new(scaling, scaling, 1.0)))
    }

    /// Returns the inverse of the [`Transformation`].
    pub fn inverse(self) -> Self {
        Self(self.0.inverse())
    }

    /// Returns the scale factor of the [`Transformation`].
    pub fn scale_factor(&self) -> f32 {
        self.0.x_axis.x
    }

    /// Returns the translation of the [`Transformation`].
    pub fn translation(&self) -> Vector {
        Vector::new(self.0.w_axis.x, self.0.w_axis.y)
    }
}

impl Default for Transformation {
    fn default() -> Self {
        Transformation::IDENTITY
    }
}

impl Mul for Transformation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self(self.0 * rhs.0)
    }
}

impl Mul<Transformation> for Point {
    type Output = Self;

    fn mul(self, transformation: Transformation) -> Self {
        let point = transformation
            .0
            .mul_vec4(Vec4::new(self.x, self.y, 1.0, 1.0));

        Point::new(point.x, point.y)
    }
}

impl Mul<Transformation> for Vector {
    type Output = Self;

    fn mul(self, transformation: Transformation) -> Self {
        let new_vector = transformation
            .0
            .mul_vec4(Vec4::new(self.x, self.y, 1.0, 0.0));

        Vector::new(new_vector.x, new_vector.y)
    }
}

impl Mul<Transformation> for Size {
    type Output = Self;

    fn mul(self, transformation: Transformation) -> Self {
        let new_size = transformation.0.mul_vec4(Vec4::new(
            self.width,
            self.height,
            1.0,
            0.0,
        ));

        Size::new(new_size.x, new_size.y)
    }
}

impl Mul<Transformation> for Rectangle {
    type Output = Self;

    fn mul(self, transformation: Transformation) -> Self {
        let position = self.position();
        let size = self.size();

        Self::new(position * transformation, size * transformation)
    }
}

impl AsRef<[f32; 16]> for Transformation {
    fn as_ref(&self) -> &[f32; 16] {
        self.0.as_ref()
    }
}

impl From<Transformation> for [f32; 16] {
    fn from(t: Transformation) -> [f32; 16] {
        *t.as_ref()
    }
}

impl From<Transformation> for Mat4 {
    fn from(transformation: Transformation) -> Self {
        transformation.0
    }
}
