//! Change the appearance of a QR code.
use crate::core::Color;

/// The appearance of a QR code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Appearance {
    /// The color of the QR code data cells
    pub cell: Color,
    /// The color of the QR code background
    pub background: Color,
}

/// A set of rules that dictate the style of a QR code.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the style of a QR code.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
