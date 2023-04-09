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

impl Default for Text {
    fn default() -> Text {
        Text {
            content: String::new(),
            position: Point::ORIGIN,
            color: Color::BLACK,
            size: 16.0,
            font: Font::Default,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }
}

impl From<String> for Text {
    fn from(content: String) -> Text {
        Text {
            content,
            ..Default::default()
        }
    }
}

impl From<&str> for Text {
    fn from(content: &str) -> Text {
        String::from(content).into()
    }
}
