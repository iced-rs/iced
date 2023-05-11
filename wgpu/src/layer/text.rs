use crate::core::alignment;
use crate::core::text;
use crate::core::{Color, Font, Rectangle};

/// A paragraph of text.
#[derive(Debug, Clone, Copy)]
pub struct Text<'a> {
    /// The content of the [`Text`].
    pub content: &'a str,

    /// The layout bounds of the [`Text`].
    pub bounds: Rectangle,

    /// The color of the [`Text`], in __linear RGB_.
    pub color: Color,

    /// The size of the [`Text`] in logical pixels.
    pub size: f32,

    /// The line height of the [`Text`].
    pub line_height: text::LineHeight,

    /// The font of the [`Text`].
    pub font: Font,

    /// The horizontal alignment of the [`Text`].
    pub horizontal_alignment: alignment::Horizontal,

    /// The vertical alignment of the [`Text`].
    pub vertical_alignment: alignment::Vertical,

    /// The shaping strategy of the text.
    pub shaping: text::Shaping,
}
