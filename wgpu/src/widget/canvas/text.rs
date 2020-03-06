use iced_native::{Color, Font, HorizontalAlignment, Rectangle, VerticalAlignment};

/// A text node to be drawn to a canvas
#[derive(Debug, Clone)]
pub struct TextNode {
    /// The contents of the text
    pub content: String,
    /// The bounds of the text
    pub bounds: Rectangle,
    /// The color of the text
    pub color: Color,
    /// The size of the text
    pub size: f32,
    /// The font of the text
    pub font: Font,
    /// The horizontal alignment of the text
    pub horizontal_alignment: HorizontalAlignment,
    /// The vertical alignment of the text
    pub vertical_alignment: VerticalAlignment,
}
