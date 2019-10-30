use crate::{Primitive, Renderer};
use iced_native::{text, Color, Layout, MouseCursor, Node, Style, Text};

use wgpu_glyph::{GlyphCruncher, Section};

use std::cell::RefCell;
use std::f32;

impl text::Renderer for Renderer {
    fn node(&self, text: &Text) -> Node {
        let glyph_brush = self.glyph_brush.clone();
        let content = text.content.clone();

        // TODO: Investigate why stretch tries to measure this MANY times
        // with every ancestor's bounds.
        // Bug? Using the library wrong? I should probably open an issue on
        // the stretch repository.
        // I noticed that the first measure is the one that matters in
        // practice. Here, we use a RefCell to store the cached measurement.
        let measure = RefCell::new(None);
        let size = text.size.map(f32::from).unwrap_or(20.0);

        let style = Style::default().width(text.width);

        iced_native::Node::with_measure(style, move |bounds| {
            let mut measure = measure.borrow_mut();

            if measure.is_none() {
                let bounds = (
                    match bounds.width {
                        iced_native::Number::Undefined => f32::INFINITY,
                        iced_native::Number::Defined(w) => w,
                    },
                    match bounds.height {
                        iced_native::Number::Undefined => f32::INFINITY,
                        iced_native::Number::Defined(h) => h,
                    },
                );

                let text = Section {
                    text: &content,
                    scale: wgpu_glyph::Scale { x: size, y: size },
                    bounds,
                    ..Default::default()
                };

                let (width, height) = if let Some(bounds) =
                    glyph_brush.borrow_mut().glyph_bounds(&text)
                {
                    (bounds.width().round(), bounds.height().round())
                } else {
                    (0.0, 0.0)
                };

                let size = iced_native::Size { width, height };

                // If the text has no width boundary we avoid caching as the
                // layout engine may just be measuring text in a row.
                if bounds.0 == f32::INFINITY {
                    return size;
                } else {
                    *measure = Some(size);
                }
            }

            measure.unwrap()
        })
    }

    fn draw(&mut self, text: &Text, layout: Layout<'_>) -> Self::Output {
        (
            Primitive::Text {
                content: text.content.clone(),
                size: f32::from(text.size.unwrap_or(20)),
                bounds: layout.bounds(),
                color: text.color.unwrap_or(Color::BLACK),
                horizontal_alignment: text.horizontal_alignment,
                vertical_alignment: text.vertical_alignment,
            },
            MouseCursor::OutOfBounds,
        )
    }
}
