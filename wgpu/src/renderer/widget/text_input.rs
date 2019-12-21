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

    fn measure_value(&self, value: &str, size: u16) -> f32 {
        let (mut width, _) = self.text_pipeline.measure(
            value,
            f32::from(size),
            Font::Default,
            Size::INFINITY,
        );

        let spaces_at_the_end = value.len() - value.trim_end().len();

        if spaces_at_the_end > 0 {
            let space_width = self.text_pipeline.space_width(size as f32);
            width += spaces_at_the_end as f32 * space_width;
        }

        width
    }

    fn offset(
        &self,
        text_bounds: Rectangle,
        size: u16,
        value: &text_input::Value,
        state: &text_input::State,
    ) -> f32 {
        if state.is_focused() {
            let (_, offset) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                state.cursor_position(value),
            );

            offset
        } else {
            0.0
        }
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
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let border = Primitive::Quad {
            bounds,
            background: Background::Color(
                if is_mouse_over || state.is_focused() {
                    [0.5, 0.5, 0.5]
                } else {
                    [0.7, 0.7, 0.7]
                }
                .into(),
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
            border_radius: 4,
        };

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
            size: f32::from(size),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
        };

        let (contents_primitive, offset) = if state.is_focused() {
            let (text_value_width, offset) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                state.cursor_position(value),
            );

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

            (
                Primitive::Group {
                    primitives: vec![text_value, cursor],
                },
                Vector::new(offset as u32, 0),
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
                primitives: vec![border, input, contents],
            },
            if is_mouse_over {
                MouseCursor::Text
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}

fn measure_cursor_and_scroll_offset(
    renderer: &Renderer,
    text_bounds: Rectangle,
    value: &text_input::Value,
    size: u16,
    cursor_index: usize,
) -> (f32, f32) {
    use iced_native::text_input::Renderer;

    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width = renderer.measure_value(&text_before_cursor, size);
    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}
