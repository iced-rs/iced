//! Display fields that can be filled with text.
//!
//! A [`TextInput`] has some local [`State`].
use crate::backend::{self, Backend};
use crate::{Font, Rectangle, Renderer, Size};

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
