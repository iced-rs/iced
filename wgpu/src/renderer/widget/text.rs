use crate::{Primitive, Renderer};
use iced_native::{layout, text, Color, Layout, MouseCursor, Size, Text};

use wgpu_glyph::{GlyphCruncher, Section};

use std::f32;

// TODO: Obtain from renderer configuration
const DEFAULT_TEXT_SIZE: f32 = 20.0;

impl text::Renderer for Renderer {
    fn layout(&self, text: &Text, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(text.width).height(text.height);
        let size = text.size.map(f32::from).unwrap_or(DEFAULT_TEXT_SIZE);
        let bounds = limits.max();

        let section = Section {
            text: &text.content,
            scale: wgpu_glyph::Scale { x: size, y: size },
            bounds: (bounds.width, bounds.height),
            ..Default::default()
        };

        let (width, height) = if let Some(bounds) =
            self.text_measurements.borrow_mut().glyph_bounds(&section)
        {
            (bounds.width().ceil(), bounds.height().ceil())
        } else {
            (0.0, 0.0)
        };

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
                horizontal_alignment: text.horizontal_alignment,
                vertical_alignment: text.vertical_alignment,
            },
            MouseCursor::OutOfBounds,
        )
    }
}
