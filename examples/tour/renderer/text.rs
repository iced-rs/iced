use super::Renderer;
use ggez::graphics::{self, mint, Align, Color, Scale, Text, TextFragment};

use iced::text;
use std::cell::RefCell;
use std::f32;

impl text::Renderer<Color> for Renderer<'_> {
    fn node(
        &self,
        style: iced::Style,
        content: &str,
        size: Option<u16>,
    ) -> iced::Node {
        let font = self.font;
        let font_cache = graphics::font_cache(self.context);
        let content = String::from(content);

        // TODO: Investigate why stretch tries to measure this MANY times
        // with every ancestor's bounds.
        // Bug? Using the library wrong? I should probably open an issue on
        // the stretch repository.
        // I noticed that the first measure is the one that matters in
        // practice. Here, we use a RefCell to store the cached measurement.
        let measure = RefCell::new(None);
        let size = size.map(f32::from).unwrap_or(self.font_size);

        iced::Node::with_measure(style, move |bounds| {
            let mut measure = measure.borrow_mut();

            if measure.is_none() {
                let bounds = (
                    match bounds.width {
                        iced::Number::Undefined => f32::INFINITY,
                        iced::Number::Defined(w) => w,
                    },
                    match bounds.height {
                        iced::Number::Undefined => f32::INFINITY,
                        iced::Number::Defined(h) => h,
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

                let size = iced::Size {
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

    fn draw(
        &mut self,
        bounds: iced::Rectangle,
        content: &str,
        size: Option<u16>,
        color: Option<Color>,
        horizontal_alignment: text::HorizontalAlignment,
        _vertical_alignment: text::VerticalAlignment,
    ) {
        let size = size.map(f32::from).unwrap_or(self.font_size);

        let mut text = Text::new(TextFragment {
            text: String::from(content),
            font: Some(self.font),
            scale: Some(Scale { x: size, y: size }),
            ..Default::default()
        });

        text.set_bounds(
            mint::Point2 {
                x: bounds.width,
                y: bounds.height,
            },
            match horizontal_alignment {
                text::HorizontalAlignment::Left => graphics::Align::Left,
                text::HorizontalAlignment::Center => graphics::Align::Center,
                text::HorizontalAlignment::Right => graphics::Align::Right,
            },
        );

        graphics::queue_text(
            self.context,
            &text,
            mint::Point2 {
                x: bounds.x,
                y: bounds.y,
            },
            color.or(Some(graphics::BLACK)),
        );
    }
}
