//! Show toggle controls using checkboxes.
use crate::alignment;
use crate::backend::{self, Backend};
use crate::{Primitive, Rectangle, Renderer};

use iced_native::checkbox;
use iced_native::mouse;

pub use iced_style::checkbox::{Style, StyleSheet};

/// A box that can be checked.
///
/// This is an alias of an `iced_native` checkbox with an `iced_wgpu::Renderer`.
pub type Checkbox<Message, Backend> =
    iced_native::Checkbox<Message, Renderer<Backend>>;

impl<B> checkbox::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_SIZE: u16 = 20;
    const DEFAULT_SPACING: u16 = 15;

    fn draw(
        &mut self,
        bounds: Rectangle,
        is_checked: bool,
        is_mouse_over: bool,
        (label, _): Self::Output,
        style_sheet: &Self::Style,
    ) -> Self::Output {
        let style = if is_mouse_over {
            style_sheet.hovered(is_checked)
        } else {
            style_sheet.active(is_checked)
        };

        let checkbox = Primitive::Quad {
            bounds,
            background: style.background,
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        };

        (
            Primitive::Group {
                primitives: if is_checked {
                    let check = Primitive::Text {
                        content: B::CHECKMARK_ICON.to_string(),
                        font: B::ICON_FONT,
                        size: bounds.height * 0.7,
                        bounds: Rectangle {
                            x: bounds.center_x(),
                            y: bounds.center_y(),
                            ..bounds
                        },
                        color: style.checkmark_color,
                        horizontal_alignment: alignment::Horizontal::Center,
                        vertical_alignment: alignment::Vertical::Center,
                    };

                    vec![checkbox, check, label]
                } else {
                    vec![checkbox, label]
                },
            },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
