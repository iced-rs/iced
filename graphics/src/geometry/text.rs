use crate::alignment;
use crate::{Color, Font, Point};

/// A bunch of text that can be drawn to a canvas
#[derive(Debug, Clone)]
pub struct Text {
    /// The contents of the text
    pub content: String,
    /// The position of the text relative to the alignment properties.
    /// By default, this position will be relative to the top-left corner coordinate meaning that
    /// if the horizontal and vertical alignments are unchanged, this property will tell where the
    /// top-left corner of the text should be placed.
    /// By changing the horizontal_alignment and vertical_alignment properties, you are are able to
    /// change what part of text is placed at this positions.
    /// For example, when the horizontal_alignment and vertical_alignment are set to Center, the
    /// center of the text will be placed at the given position NOT the top-left coordinate.
    pub position: Point,
    /// The color of the text
    pub color: Color,
    /// The size of the text
    pub size: f32,
    /// The font of the text
    pub font: Font,
    /// The horizontal alignment of the text
    pub horizontal_alignment: alignment::Horizontal,
    /// The vertical alignment of the text
    pub vertical_alignment: alignment::Vertical,
}

impl Default for Text {
    fn default() -> Text {
        Text {
            content: String::new(),
            position: Point::ORIGIN,
            color: Color::BLACK,
            size: 16.0,
            font: Font::SansSerif,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
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
