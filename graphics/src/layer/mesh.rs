//! A collection of triangle primitives.

use crate::{Color, Point, Rectangle, triangle};
use crate::gradient::Gradient;

/// A mesh of triangles.
#[derive(Debug, Clone, Copy)]
pub struct Mesh<'a> {
    /// The origin of the vertices of the [`Mesh`].
    pub origin: Point,

    /// The vertex and index buffers of the [`Mesh`].
    pub buffers: &'a triangle::Mesh2D,

    /// The clipping bounds of the [`Mesh`].
    pub clip_bounds: Rectangle<f32>,

    /// The shader of the [`Mesh`].
    pub style: &'a Style,
}

#[derive(Debug, Clone)]
/// Supported shaders for primitives.
pub enum Style {
    /// Fill a primitive with a solid color.
    Solid(Color),
    /// Fill a primitive with an interpolated color.
    Gradient(Gradient)
}

impl <'a> Into<Style> for Gradient {
    fn into(self) -> Style {
        match self {
            Gradient::Linear(linear) => {
                Style::Gradient(Gradient::Linear(linear))
            }
        }
    }
}