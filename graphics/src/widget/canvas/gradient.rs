//! Define a color gradient.
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
    pub(super) fn pattern(&self) -> Pattern {
        match self {
            Gradient::Linear(linear) => Pattern::Gradient(linear.gradient()),
        }
    }
}
