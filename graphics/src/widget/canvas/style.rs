use crate::{Color, Gradient};

/// The coloring style of some drawing.
#[derive(Debug, Clone, PartialEq)]
pub enum Style {
    /// A solid [`Color`].
    Solid(Color),

    /// A [`Gradient`] color.
    Gradient(Gradient),
}
