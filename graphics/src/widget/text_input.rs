//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use crate::alignment;
use crate::backend::{self, Backend};
use crate::{
    Background, Color, Font, Point, Primitive, Rectangle, Renderer, Size,
    Vector,
};

use iced_native::mouse;
use iced_native::text_input::{self, cursor};
use std::f32;

pub use iced_native::text_input::State;
pub use iced_style::text_input::{Style, StyleSheet};

/// A field that can be filled with text.
///
/// This is an alias of an `iced_native` text input with an `iced_wgpu::Renderer`.
pub type TextInput<'a, Message, Backend> =
    iced_native::TextInput<'a, Message, Renderer<Backend>>;

impl<B> text_input::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    fn measure_value(&self, value: &str, size: u16, font: Font) -> f32 {
        let backend = self.backend();

        let (width, _) =
            backend.measure(value, f32::from(size), font, Size::INFINITY);

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
            },
            font,
            bounds: Rectangle {
                y: text_bounds.center_y(),
                width: f32::INFINITY,
                ..text_bounds
            },
            size: f32::from(size),
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Center,
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
                            border_radius: 0.0,
                            border_width: 0.0,
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
                            border_radius: 0.0,
                            border_width: 0.0,
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
                    primitives: vec![cursor_primitive, text_value],
                },
                Vector::new(offset as u32, 0),
            )
        } else {
            (text_value, Vector::new(0, 0))
        };

        let text_width = self.measure_value(
            if text.is_empty() { placeholder } else { &text },
            size,
            font,
        );

        let contents = if text_width > text_bounds.width {
            Primitive::Clip {
                bounds: text_bounds,
                offset,
                content: Box::new(contents_primitive),
            }
        } else {
            contents_primitive
        };

        (
            Primitive::Group {
                primitives: vec![input, contents],
            },
            if is_mouse_over {
                mouse::Interaction::Text
            } else {
                mouse::Interaction::default()
            },
        )
    }
}

fn measure_cursor_and_scroll_offset<B>(
    renderer: &Renderer<B>,
    text_bounds: Rectangle,
    value: &text_input::Value,
    size: u16,
    cursor_index: usize,
    font: Font,
) -> (f32, f32)
where
    B: Backend + backend::Text,
{
    use iced_native::text_input::Renderer;

    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_value(&text_before_cursor, size, font);
    let offset = ((text_value_width + 5.0) - text_bounds.width).max(0.0);

    (text_value_width, offset)
}
