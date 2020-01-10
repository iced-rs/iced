use crate::{checkbox::StyleSheet, Primitive, Renderer};
use iced_native::{
    checkbox, HorizontalAlignment, MouseCursor, Rectangle, VerticalAlignment,
};

const SIZE: f32 = 28.0;

impl checkbox::Renderer for Renderer {
    type Style = Box<dyn StyleSheet>;

    fn default_size(&self) -> u32 {
        SIZE as u32
    }

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
                        content: crate::text::CHECKMARK_ICON.to_string(),
                        font: crate::text::BUILTIN_ICONS,
                        size: bounds.height * 0.7,
                        bounds: bounds,
                        color: style.checkmark_color,
                        horizontal_alignment: HorizontalAlignment::Center,
                        vertical_alignment: VerticalAlignment::Center,
                    };

                    vec![checkbox, check, label]
                } else {
                    vec![checkbox, label]
                },
            },
            if is_mouse_over {
                MouseCursor::Pointer
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
