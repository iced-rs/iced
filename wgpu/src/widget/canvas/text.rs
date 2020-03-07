use iced_native::{Color, Font, HorizontalAlignment, Point, VerticalAlignment};

/// A bunch of text that can be drawn to a canvas
#[derive(Debug, Clone)]
pub struct Text {
    /// The contents of the text
    pub content: String,
    /// The position where to begin drawing the text (top-left corner coordinates)
    pub position: Point,
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
