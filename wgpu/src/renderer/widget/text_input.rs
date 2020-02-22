use crate::{text_input::StyleSheet, Primitive, Renderer};

use iced_native::{
    text_input, Background, Color, Font, HorizontalAlignment, MouseCursor,
    Point, Rectangle, Size, Vector, VerticalAlignment,
};
use std::f32;

impl text_input::Renderer for Renderer {
    type Style = Box<dyn StyleSheet>;

    fn default_size(&self) -> u16 {
        // TODO: Make this configurable
        20
    }

    fn measure_value(&self, value: &str, size: u16, font: Font) -> f32 {
        let (mut width, _) = self.text_pipeline.measure(
            value,
            f32::from(size),
            font,
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
        font: Font,
    ) -> f32 {
        if state.is_focused() {
            let (_, offset) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                state.cursor_position(value).position(),
                font,
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
        font: Font,
        placeholder: &str,
        value: &text_input::Value,
        state: &text_input::State,
        style_sheet: &Self::Style,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let style = if state.is_focused() {
            style_sheet.focused()
        } else if is_mouse_over {
            style_sheet.hovered()
        } else {
            style_sheet.active()
        };

        let input = Primitive::Quad {
            bounds,
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        };

        let text = value.to_string();

        let text_value = Primitive::Text {
            content: if text.is_empty() {
                placeholder.to_string()
            } else {
                text.clone()
            },
            color: if text.is_empty() {
                style_sheet.placeholder_color()
            } else {
                style_sheet.value_color()
            }
            .into(),
            font,
            bounds: Rectangle {
                width: f32::INFINITY,
                ..text_bounds
            },
            size: f32::from(size),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
        };

        let (contents_primitive, offset) = if state.is_focused() {
            let cursor = state.cursor_position(value);
            let (text_value_width, offset) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                cursor.position(),
                font,
            );

            let selection = match cursor {
                text_input::Cursor::Index(_) => Primitive::None,
                text_input::Cursor::Selection { .. } => {
                    let (cursor_left_offset, _) =
                        measure_cursor_and_scroll_offset(
                            self,
                            text_bounds,
                            value,
                            size,
                            cursor.left(),
                            font,
                        );
                    let (cursor_right_offset, _) =
                        measure_cursor_and_scroll_offset(
                            self,
                            text_bounds,
                            value,
                            size,
                            cursor.right(),
                            font,
                        );
                    let width = cursor_right_offset - cursor_left_offset;
                    Primitive::Quad {
                        bounds: Rectangle {
                            x: text_bounds.x + cursor_left_offset,
                            y: text_bounds.y,
                            width,
                            height: text_bounds.height,
                        },
                        background: Background::Color(
                            style_sheet.selection_color(),
                        ),
                        border_radius: 0,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    }
                }
            };

            let cursor = Primitive::Quad {
                bounds: Rectangle {
                    x: text_bounds.x + text_value_width,
                    y: text_bounds.y,
                    width: 1.0,
                    height: text_bounds.height,
                },
                background: Background::Color(style_sheet.value_color()),
                border_radius: 0,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            };

            (
                Primitive::Group {
                    primitives: vec![selection, text_value, cursor],
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

fn measure_cursor_and_scroll_offset(
    renderer: &Renderer,
    text_bounds: Rectangle,
    value: &text_input::Value,
    size: u16,
    cursor_index: usize,
    font: Font,
) -> (f32, f32) {
    use iced_native::text_input::Renderer;

    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_value(&text_before_cursor, size, font);
    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}
