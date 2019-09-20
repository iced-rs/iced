use super::{into_color, Renderer};
use ggez::graphics::{self, mint, Align, Scale, Text, TextFragment};

use iced_native::{text, Layout, Node, Style};
use std::cell::RefCell;
use std::f32;

impl text::Renderer for Renderer<'_> {
    fn node(&self, text: &iced_native::Text) -> Node {
        let font = self.font;
        let font_cache = graphics::font_cache(self.context);
        let content = String::from(&text.content);

        // TODO: Investigate why stretch tries to measure this MANY times
        // with every ancestor's bounds.
        // Bug? Using the library wrong? I should probably open an issue on
        // the stretch repository.
        // I noticed that the first measure is the one that matters in
        // practice. Here, we use a RefCell to store the cached measurement.
        let measure = RefCell::new(None);
        let size = text.size.map(f32::from).unwrap_or(self.font_size);

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

                let mut text = Text::new(TextFragment {
                    text: content.clone(),
                    font: Some(font),
                    scale: Some(Scale { x: size, y: size }),
                    ..Default::default()
                });

                text.set_bounds(
                    mint::Point2 {
                        x: bounds.0,
                        y: bounds.1,
                    },
                    Align::Left,
                );

                let (width, height) = font_cache.dimensions(&text);

                let size = iced_native::Size {
                    width: width as f32,
                    height: height as f32,
                };

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

    fn draw(&mut self, text: &iced_native::Text, layout: Layout<'_>) {
        let size = text.size.map(f32::from).unwrap_or(self.font_size);
        let bounds = layout.bounds();

        let mut ggez_text = Text::new(TextFragment {
            text: text.content.clone(),
            font: Some(self.font),
            scale: Some(Scale { x: size, y: size }),
            ..Default::default()
        });

        ggez_text.set_bounds(
            mint::Point2 {
                x: bounds.width,
                y: bounds.height,
            },
            match text.horizontal_alignment {
                text::HorizontalAlignment::Left => graphics::Align::Left,
                text::HorizontalAlignment::Center => graphics::Align::Center,
                text::HorizontalAlignment::Right => graphics::Align::Right,
            },
        );

        graphics::queue_text(
            self.context,
            &ggez_text,
            mint::Point2 {
                x: bounds.x,
                y: bounds.y,
            },
            text.color
                .or(Some(iced_native::Color::BLACK))
                .map(into_color),
        );
    }
}
