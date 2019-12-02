use crate::{Primitive, Renderer};

use iced_native::{
    text_input, Background, Color, Font, HorizontalAlignment, MouseCursor,
    Point, Rectangle, Size, Vector, VerticalAlignment,
};
use std::f32;

impl text_input::Renderer for Renderer {
    fn default_size(&self) -> u16 {
        // TODO: Make this configurable
        20
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
        size: u16,
        placeholder: &str,
        value: &text_input::Value,
        state: &text_input::State,
        border_radius: u16,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let input = Primitive::Quad {
            bounds,
            background: Background::Color(Color::WHITE),
            border_radius,
            border_color: if is_mouse_over || state.is_focused() {
                [0.5, 0.5, 0.5].into()
            } else {
                [0.7, 0.7, 0.7].into()
            },
            border_width: 1,
        };

        let size = f32::from(size);
        let text = value.to_string();

        let text_value = Primitive::Text {
            content: if text.is_empty() {
                placeholder.to_string()
            } else {
                text.clone()
            },
            color: if text.is_empty() {
                [0.7, 0.7, 0.7]
            } else {
                [0.3, 0.3, 0.3]
            }
            .into(),
            font: Font::Default,
            bounds: Rectangle {
                width: f32::INFINITY,
                ..text_bounds
            },
            size,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
        };

        let (contents_primitive, offset) = if state.is_focused() {
            let text_before_cursor =
                value.until(state.cursor_position(value)).to_string();

            let (mut text_value_width, _) = self.text_pipeline.measure(
                &text_before_cursor,
                size,
                Font::Default,
                Size::new(f32::INFINITY, text_bounds.height),
            );

            let spaces_at_the_end =
                text_before_cursor.len() - text_before_cursor.trim_end().len();

            if spaces_at_the_end > 0 {
                let space_width = self.text_pipeline.space_width(size);
                text_value_width += spaces_at_the_end as f32 * space_width;
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
                border_color: Color::BLACK,
                border_width: 0,
            };

            (
                Primitive::Group {
                    primitives: vec![text_value, cursor],
                },
                Vector::new(
                    ((text_value_width + 5.0) - text_bounds.width).max(0.0)
                        as u32,
                    0,
                ),
            )
        } else {
            (text_value, Vector::new(0, 0))
        };

        let contents = Primitive::Clip {
            bounds: text_bounds,
            offset,
            content: Box::new(contents_primitive),
        };

        (
            Primitive::Group {
                primitives: vec![input, contents],
            },
            if is_mouse_over {
                MouseCursor::Text
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
