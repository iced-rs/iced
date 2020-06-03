//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
//!
//! [`TextInput`]: struct.TextInput.html
//! [`State`]: struct.State.html
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::mouse;
use iced_native::text_input::{self, cursor};
use iced_native::{
    Background, Color, Font, HorizontalAlignment, Point, Rectangle, Size,
    Vector, VerticalAlignment,
};
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
    type Font = Font;
    type Style = Box<dyn StyleSheet>;

    fn default_size(&self) -> u16 {
        // TODO: Make this configurable
        20
    }

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
        horizontal_alignment: HorizontalAlignment,
    ) -> f32 {
        if state.is_focused() {
            let cursor = state.cursor();

            let focus_position = match cursor.state(value) {
                cursor::State::Index(i) => i,
                cursor::State::Selection { end, .. } => end,
            };

            let (_, offset, _) = measure_cursor_and_scroll_offset(
                self,
                text_bounds,
                value,
                size,
                focus_position,
                font,
                horizontal_alignment,
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
        horizontal_alignment: HorizontalAlignment,
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

        let text_width = self.measure_value(
            if text.is_empty() { placeholder } else { &text },
            size,
            font,
        );

        let current_text_bounds = Rectangle {
            x: match horizontal_alignment {
                HorizontalAlignment::Left => text_bounds.x,
                HorizontalAlignment::Center => text_bounds.center_x(),
                HorizontalAlignment::Right => text_bounds.x + text_bounds.width,
            },
            y: text_bounds.center_y(),
            width: f32::INFINITY,
            ..text_bounds
        };

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
            bounds: current_text_bounds,
            size: f32::from(size),
            horizontal_alignment,
            vertical_alignment: VerticalAlignment::Center,
        };

        let (contents_primitive, offset) = if state.is_focused() {
            let cursor = state.cursor();

            let (cursor_primitive, offset) = match cursor.state(value) {
                cursor::State::Index(position) => {
                    let (text_value_width, offset, text_width_after_cursor) =
                        measure_cursor_and_scroll_offset(
                            self,
                            text_bounds,
                            value,
                            size,
                            position,
                            font,
                            horizontal_alignment,
                        );

                    let cursor_bounds = Rectangle {
                        x: match horizontal_alignment {
                            HorizontalAlignment::Left => {
                                text_bounds.x + text_value_width
                            }
                            HorizontalAlignment::Center => {
                                text_bounds.center_x()
                                    + (text_value_width / 2.0)
                                    - (text_width_after_cursor / 2.0)
                            }
                            HorizontalAlignment::Right => {
                                text_bounds.x + text_bounds.width
                                    - text_width_after_cursor
                            }
                        },
                        width: f32::INFINITY,
                        ..text_bounds
                    };

                    (
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: cursor_bounds.x,
                                y: cursor_bounds.y,
                                width: 1.0,
                                height: cursor_bounds.height,
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

                    let selection_bounds = Rectangle {
                        x: match horizontal_alignment {
                            HorizontalAlignment::Left => text_bounds.x,
                            HorizontalAlignment::Center => {
                                text_bounds.center_x()
                            }
                            HorizontalAlignment::Right => {
                                text_bounds.x + text_bounds.width
                            }
                        },
                        width: f32::INFINITY,
                        ..text_bounds
                    };

                    let (left_position, left_offset, _) =
                        measure_cursor_and_scroll_offset(
                            self,
                            selection_bounds,
                            value,
                            size,
                            left,
                            font,
                            horizontal_alignment,
                        );

                    let (right_position, right_offset, _) =
                        measure_cursor_and_scroll_offset(
                            self,
                            selection_bounds,
                            value,
                            size,
                            right,
                            font,
                            horizontal_alignment,
                        );

                    let width = right_position - left_position;
                    let aligned_left_position = match horizontal_alignment {
                        HorizontalAlignment::Left => left_position,
                        HorizontalAlignment::Center => {
                            left_position - text_width / 2.0
                        }
                        HorizontalAlignment::Right => {
                            left_position - text_width
                        }
                    };

                    (
                        Primitive::Quad {
                            bounds: Rectangle {
                                x: selection_bounds.x + aligned_left_position,
                                y: selection_bounds.y,
                                width,
                                height: selection_bounds.height,
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
                    primitives: vec![cursor_primitive, text_value],
                },
                Vector::new(offset as u32, 0),
            )
        } else {
            (text_value, Vector::new(0, 0))
        };

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
    horizontal_alignment: HorizontalAlignment,
) -> (f32, f32, f32)
where
    B: Backend + backend::Text,
{
    use iced_native::text_input::Renderer;

    let text_before_cursor = value.until(cursor_index).to_string();

    let text_value_width =
        renderer.measure_value(&text_before_cursor, size, font);
    let text_width = renderer.measure_value(&value.to_string(), size, font);

    let offset = {
        let offset = match horizontal_alignment {
            HorizontalAlignment::Left => {
                (text_value_width + 5.0) - text_bounds.width
            }
            HorizontalAlignment::Center => {
                ((text_value_width + 5.0) - text_bounds.width) / 2.0
            }
            HorizontalAlignment::Right => 0.0,
        };
        offset.max(0.0)
    };

    (text_value_width, offset, text_width - text_value_width)
}
