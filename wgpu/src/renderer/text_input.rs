use crate::{Primitive, Renderer};

use iced_native::{
    text::HorizontalAlignment, text::VerticalAlignment, text_input, Background,
    Color, MouseCursor, Point, Rectangle, TextInput,
};
use std::f32;

impl text_input::Renderer for Renderer {
    fn default_size(&self) -> u16 {
        // TODO: Make this configurable
        20
    }

    fn draw<Message>(
        &mut self,
        text_input: &TextInput<Message>,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let border = Primitive::Quad {
            bounds,
            background: Background::Color(
                if is_mouse_over || text_input.state.is_focused {
                    Color {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.7,
                        g: 0.7,
                        b: 0.7,
                        a: 1.0,
                    }
                },
            ),
            border_radius: 5,
        };

        let input = Primitive::Quad {
            bounds: Rectangle {
                x: bounds.x + 1.0,
                y: bounds.y + 1.0,
                width: bounds.width - 2.0,
                height: bounds.height - 2.0,
            },
            background: Background::Color(Color::WHITE),
            border_radius: 5,
        };

        let size = f32::from(text_input.size.unwrap_or(self.default_size()));
        let text = text_input.value.to_string();

        let value = Primitive::Clip {
            bounds: text_bounds,
            offset: 0,
            content: Box::new(Primitive::Text {
                content: if text.is_empty() {
                    text_input.placeholder.clone()
                } else {
                    text.clone()
                },
                color: if text.is_empty() {
                    Color {
                        r: 0.7,
                        g: 0.7,
                        b: 0.7,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.3,
                        g: 0.3,
                        b: 0.3,
                        a: 1.0,
                    }
                },
                bounds: Rectangle {
                    width: f32::INFINITY,
                    ..text_bounds
                },
                size,
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
            }),
        };

        (
            Primitive::Group {
                primitives: if text_input.state.is_focused {
                    use wgpu_glyph::{GlyphCruncher, Scale, Section};

                    let text_before_cursor = &text_input
                        .value
                        .until(
                            text_input.state.cursor_position(&text_input.value),
                        )
                        .to_string();

                    let mut text_value_width = self
                        .glyph_brush
                        .borrow_mut()
                        .glyph_bounds(Section {
                            text: text_before_cursor,
                            bounds: (f32::INFINITY, text_bounds.height),
                            scale: Scale { x: size, y: size },
                            ..Default::default()
                        })
                        .map(|bounds| bounds.width().round())
                        .unwrap_or(0.0);

                    let spaces_at_the_end = text_before_cursor.len()
                        - text_before_cursor.trim_end().len();

                    if spaces_at_the_end > 0 {
                        let space_width = {
                            let glyph_brush = self.glyph_brush.borrow();

                            // TODO: Select appropriate font
                            let font = &glyph_brush.fonts()[0];

                            font.glyph(' ')
                                .scaled(Scale { x: size, y: size })
                                .h_metrics()
                                .advance_width
                        };

                        text_value_width +=
                            spaces_at_the_end as f32 * space_width;
                    }

                    let cursor = Primitive::Quad {
                        bounds: Rectangle {
                            x: text_bounds.x + text_value_width,
                            y: text_bounds.y,
                            width: 1.0,
                            height: text_bounds.height,
                        },
                        background: Background::Color(Color::BLACK),
                        border_radius: 0,
                    };

                    vec![border, input, value, cursor]
                } else {
                    vec![border, input, value]
                },
            },
            if is_mouse_over {
                MouseCursor::Text
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
