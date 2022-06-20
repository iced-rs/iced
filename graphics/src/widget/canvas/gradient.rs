//! Define a color gradient.
use iced_native::Point;

use crate::pattern::Pattern;

pub mod linear;

pub use linear::Linear;

/// A gradient that can be used in the style of [`super::Fill`] or [`super::Stroke`].
#[derive(Debug, Clone)]
pub enum Gradient {
    /// A linear gradient
    Linear(Linear),
}

impl Gradient {
    /// Creates a new linear [`linear::Builder`].
    pub fn linear(start: Point, end: Point) -> linear::Builder {
        linear::Builder::new(start, end)
    }

    pub(super) fn pattern(&self) -> Pattern {
        match self {
            Gradient::Linear(linear) => Pattern::Gradient(linear.gradient()),
        }
    }
}
