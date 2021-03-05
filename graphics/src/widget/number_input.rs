//! Display fields that can only be filled with numeric type.
//!
//! A [`NumberInput`] has some local [`State`].
use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::mouse;
use iced_native::number_input::{self, ModifierState};
use iced_native::{
    Background, Color, HorizontalAlignment, Point, Rectangle, VerticalAlignment,
};

pub use iced_native::number_input::State;
pub use iced_style::number_input::{Style, StyleSheet};

/// A field that can only be filled with numeric type.
///
/// This is an alias of an `iced_native` number input with an `iced_wgpu::Renderer`.
pub type NumberInput<'a, T, Message, Backend> =
    iced_native::NumberInput<'a, T, Message, Renderer<Backend>>;

impl<B> number_input::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_PADDING: u16 = 5;

    fn draw(
        &mut self,
        cursor_position: Point,
        state: &ModifierState,
        inc_bounds: Rectangle,
        dec_bounds: Rectangle,
        is_mouse_over: bool,
        is_decrease_disabled: bool,
        is_increase_disabled: bool,
        (content, _): Self::Output,
        style: &<Self as number_input::Renderer>::Style,
        font: Self::Font,
    ) -> Self::Output {
        let mouse_over_decrease = dec_bounds.contains(cursor_position);
        let mouse_over_increase = inc_bounds.contains(cursor_position);

        let decrease_btn_style = if is_decrease_disabled {
            style.disabled()
        } else if state.decrease_pressed {
            style.pressed()
        } else {
            style.active()
        };

        let increase_btn_style = if is_increase_disabled {
            style.disabled()
        } else if state.increase_pressed {
            style.pressed()
        } else {
            style.active()
        };

        // decrease button section
        let decrease_button_rect = Primitive::Quad {
            bounds: dec_bounds,
            background: decrease_btn_style
                .button_background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
            border_radius: 3.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        };
        let decrease_text = Primitive::Text {
            content: String::from("▼"),
            bounds: Rectangle {
                x: dec_bounds.center_x(),
                y: dec_bounds.center_y(),
                ..dec_bounds
            },
            font,
            size: dec_bounds.height * 0.9,
            color: decrease_btn_style.icon_color,
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
        };
        let decrease_btn = Primitive::Group {
            primitives: vec![decrease_button_rect, decrease_text],
        };

        // increase button section
        let increase_button_rect = Primitive::Quad {
            bounds: inc_bounds,
            background: increase_btn_style
                .button_background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
            border_radius: 3.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        };
        let increase_text = Primitive::Text {
            content: String::from("▲"),
            bounds: Rectangle {
                x: inc_bounds.center_x(),
                y: inc_bounds.center_y(),
                ..inc_bounds
            },
            font,
            size: inc_bounds.height * 0.9,
            color: increase_btn_style.icon_color,
            horizontal_alignment: HorizontalAlignment::Center,
            vertical_alignment: VerticalAlignment::Center,
        };
        let increase_btn = Primitive::Group {
            primitives: vec![increase_button_rect, increase_text],
        };

        (
            Primitive::Group {
                primitives: vec![content, decrease_btn, increase_btn],
            },
            if (mouse_over_decrease && !is_decrease_disabled)
                || (mouse_over_increase && !is_increase_disabled)
            {
                mouse::Interaction::Pointer
            } else if is_mouse_over {
                mouse::Interaction::Text
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
