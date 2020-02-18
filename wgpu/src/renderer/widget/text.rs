use crate::{Primitive, Renderer};
use iced_native::{
    text, Color, Font, HorizontalAlignment, MouseCursor, Rectangle, Size,
    VerticalAlignment,
};

use std::f32;

impl text::Renderer for Renderer {
    const DEFAULT_SIZE: u16 = 20;

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
        defaults: &Self::Defaults,
        bounds: Rectangle,
        content: &str,
        size: u16,
        font: Font,
        color: Option<Color>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self::Output {
        (
            Primitive::Text {
                content: content.to_string(),
                size: f32::from(size),
                bounds,
                color: color.unwrap_or(defaults.text.color),
                font,
                horizontal_alignment,
                vertical_alignment,
            },
            MouseCursor::OutOfBounds,
        )
    }
}
