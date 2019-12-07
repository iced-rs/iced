use crate::{Primitive, Renderer, TextStyle};
use iced_native::{
    text, Font, HorizontalAlignment, MouseCursor, Rectangle, Size,
    VerticalAlignment,
};

use std::f32;

impl text::Renderer for Renderer {
    type WidgetStyle = TextStyle;

    fn measure(
        &self,
        content: &str,
        size: u16,
        font: Font,
        bounds: Size,
    ) -> (f32, f32) {
        self.text_pipeline
            .measure(content, f32::from(size), font, bounds)
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        content: &str,
        size: u16,
        font: Font,
        style: &TextStyle,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self::Output {
        (
            Primitive::Text {
                content: content.to_string(),
                size: f32::from(size),
                bounds,
                color: style.text_color,
                font,
                horizontal_alignment,
                vertical_alignment,
            },
            MouseCursor::OutOfBounds,
        )
    }
}
