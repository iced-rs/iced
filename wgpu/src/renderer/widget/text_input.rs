use crate::{text_input::StyleSheet, Primitive, Renderer};

use iced_native::{
    text_input::{self, cursor},
    Background, Color, Depth, Font, HorizontalAlignment, MouseCursor, Point,
    Rectangle, Size, Vector, VerticalAlignment,
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
        font: Font,
        size: u16,
        value: &text_input::Value,
        state: &text_input::State,
    ) -> f32 {
        if state.is_focused() {
            let cursor = state.cursor();

            let focus_position = match cursor.state(value) {
                cursor::State::Index(i) => i,
                cursor::State::Selection { end, .. } => end,
            };

            let (_, offset) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                focus_position,
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
        font: Font,
        size: u16,
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
            let cursor = state.cursor();

            let (cursor_primitive, offset) = match cursor.state(value) {
                cursor::State::Index(position) => {
                    let (text_value_width, offset) =
                        measure_cursor_and_scroll_offset(
                            self,
                            text_bounds,
                            value,
                            size,
                            position,
                            font,
                        );

                    (
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: text_bounds.x + text_value_width,
                                y: text_bounds.y,
                                width: 1.0,
                                height: text_bounds.height,
                            },
                            background: Background::Color(
                                style_sheet.value_color(),
                            ),
                            border_radius: 0,
                            border_width: 0,
                            border_color: Color::TRANSPARENT,
                        },
                        offset,
                    )
                }
                cursor::State::Selection { start, end } => {
                    let left = start.min(end);
                    let right = end.max(start);

                    let (left_position, left_offset) =
                        measure_cursor_and_scroll_offset(
                            self,
                            text_bounds,
                            value,
                            size,
                            left,
                            font,
                        );

                    let (right_position, right_offset) =
                        measure_cursor_and_scroll_offset(
                            self,
                            text_bounds,
                            value,
                            size,
                            right,
                            font,
                        );

                    let width = right_position - left_position;

                    (
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: text_bounds.x + left_position,
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
                        },
                        if end == right {
                            right_offset
                        } else {
                            left_offset
                        },
                    )
                }
            };

            (
                Primitive::Group {
                    primitives: vec![
                        (cursor_primitive, Depth::None),
                        (text_value, Depth::None),
                    ],
                },
                Vector::new(offset as u32, 0),
            )
        } else {
            (text_value, Vector::new(0, 0))
        };

        let contents = Primitive::Clip {
            bounds: text_bounds,
            offset,
            content: Box::new((contents_primitive, Depth::None)),
        };

        (
            (
                Primitive::Group {
                    primitives: vec![
                        (input, Depth::None),
                        (contents, Depth::None),
                    ],
                },
                Depth::None,
            ),
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
