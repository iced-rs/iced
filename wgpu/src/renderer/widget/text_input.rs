use crate::{Primitive, Renderer, TextInputStyle};

use iced_native::{
    text_input, Background, Color, HorizontalAlignment, MouseCursor, Point,
    Rectangle, Size, Vector, VerticalAlignment,
};
use std::f32;

impl text_input::Renderer for Renderer {
    type WidgetStyle = TextInputStyle;

    fn default_size(&self) -> u16 {
        // TODO: Make this configurable
        20
    }

    fn measure_value(
        &self,
        value: &str,
        size: u16,
        style: &Self::WidgetStyle,
    ) -> f32 {
        let (mut width, _) = self.text_pipeline.measure(
            value,
            f32::from(size),
            style.font,
            Size::INFINITY,
        );

        let spaces_at_the_end = value.len() - value.trim_end().len();

        if spaces_at_the_end > 0 {
            let space_width = self.text_pipeline.space_width(size as f32);
            width += spaces_at_the_end as f32 * space_width;
        }

        width
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        text_bounds: Rectangle,
        cursor_position: Point,
        style: &TextInputStyle,
        size: u16,
        placeholder: &str,
        value: &text_input::Value,
        state: &text_input::State,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let border_color = if is_mouse_over || state.is_focused() {
            style.get_border_hovered_color()
        } else {
            style.border_color
        };

        let border = Primitive::Quad {
            bounds,
            background: Background::Color(border_color),
            border_radius: style.border_radius,
        };

        let input = Primitive::Quad {
            bounds: Rectangle {
                x: bounds.x + f32::from(style.border_width),
                y: bounds.y + f32::from(style.border_width),
                width: bounds.width - f32::from(style.border_width * 2),
                height: bounds.height - f32::from(style.border_width * 2),
            },
            background: if let Some(background) = style.background {
                background
            } else {
                Background::Color(Color::WHITE)
            },
            border_radius: style.border_radius - style.border_width,
        };

        let text = value.to_string();

        let text_value = Primitive::Text {
            content: if text.is_empty() {
                placeholder.to_string()
            } else {
                text.clone()
            },
            color: if text.is_empty() {
                style.placeholder_color
            } else {
                style.text_color
            },
            font: style.font,
            bounds: Rectangle {
                width: f32::INFINITY,
                ..text_bounds
            },
            size: f32::from(size),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
        };

        let (contents_primitive, offset) = if state.is_focused() {
            let text_before_cursor =
                value.until(state.cursor_position(value)).to_string();

            let text_value_width =
                self.measure_value(&text_before_cursor, size, style);

            let cursor = Primitive::Quad {
                bounds: Rectangle {
                    x: text_bounds.x + text_value_width,
                    y: text_bounds.y,
                    width: 1.0,
                    height: text_bounds.height,
                },
                background: Background::Color(style.text_color),
                border_radius: 1,
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
