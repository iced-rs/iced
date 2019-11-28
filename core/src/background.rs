use crate::Color;

/// The background of some element.
#[derive(Debug, Clone, PartialEq)]
pub enum Background {
    /// A solid color
    Color(Color),
    /// A gradient color
    Gradient(Vec<Color>),
    // TODO: Add gradient and image variants
}
