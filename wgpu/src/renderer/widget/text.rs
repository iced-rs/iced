use crate::{Primitive, Renderer};
use iced_native::{layout, text, Color, Layout, MouseCursor, Size, Text};

use std::f32;

// TODO: Obtain from renderer configuration
const DEFAULT_TEXT_SIZE: f32 = 20.0;

impl text::Renderer for Renderer {
    fn layout(&self, text: &Text, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(text.width).height(text.height);
        let size = text.size.map(f32::from).unwrap_or(DEFAULT_TEXT_SIZE);
        let bounds = limits.max();

        let (width, height) =
            self.text_pipeline
                .measure(&text.content, size, text.font, bounds);

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn draw(&mut self, text: &Text, layout: Layout<'_>) -> Self::Output {
        (
            Primitive::Text {
                content: text.content.clone(),
                size: text.size.map(f32::from).unwrap_or(DEFAULT_TEXT_SIZE),
                bounds: layout.bounds(),
                color: text.color.unwrap_or(Color::BLACK),
                font: text.font,
                horizontal_alignment: text.horizontal_alignment,
                vertical_alignment: text.vertical_alignment,
            },
            MouseCursor::OutOfBounds,
        )
    }
}
