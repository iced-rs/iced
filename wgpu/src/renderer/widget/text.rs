use crate::{Primitive, Renderer};
use iced_native::{layout, text, Color, Layout, MouseCursor, Rectangle, Text};

//use wgpu_glyph::{GlyphCruncher, Section};

use std::f32;

// TODO: Obtain from renderer configuration
const DEFAULT_TEXT_SIZE: f32 = 20.0;

impl text::Renderer for Renderer {
    fn layout(&self, text: &Text, limits: &layout::Limits) -> Layout {
        // TODO
        Layout::new(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        })
    }

    fn draw(&mut self, text: &Text, layout: &Layout) -> Self::Output {
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
