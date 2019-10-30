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

        let value = Primitive::Clip {
            bounds: text_bounds,
            offset: 0,
            content: Box::new(Primitive::Text {
                content: if text_input.value.is_empty() {
                    text_input.placeholder.clone()
                } else {
                    text_input.value.clone()
                },
                color: if text_input.value.is_empty() {
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
                size: f32::from(text_input.size.unwrap_or(self.default_size())),
                horizontal_alignment: HorizontalAlignment::Left,
                vertical_alignment: VerticalAlignment::Center,
            }),
        };

        (
            Primitive::Group {
                primitives: vec![border, input, value],
            },
            if is_mouse_over {
                MouseCursor::Text
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
