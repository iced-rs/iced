use crate::{CheckboxStyle, Primitive, Renderer};
use iced_native::{
    checkbox, Background, HorizontalAlignment, MouseCursor, Rectangle,
    VerticalAlignment,
};

const SIZE: f32 = 28.0;

impl checkbox::Renderer for Renderer {
    type WidgetStyle = CheckboxStyle;

    fn default_size(&self) -> u32 {
        SIZE as u32
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        is_checked: bool,
        is_mouse_over: bool,
        style: &Self::WidgetStyle,
        (label, _): Self::Output,
    ) -> Self::Output {
        let (checkbox_border, checkbox_box) = (
            Primitive::Quad {
                bounds,
                background: Background::Color(
                    if is_mouse_over {
                        style.get_border_hovered_color()
                    } else {
                        style.border_color
                    }
                ),
                border_radius: style.border_radius,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x + style.border_width as f32,
                    y: bounds.y + style.border_width as f32,
                    width: bounds.width - (style.border_width * 2) as f32,
                    height: bounds.height - (style.border_width * 2) as f32,
                },
                background: style.background,
                border_radius: (style.border_radius - 1).max(0),
            },
        );

        (
            Primitive::Group {
                primitives: if is_checked {
                    let check = Primitive::Text {
                        content: crate::text::CHECKMARK_ICON.to_string(),
                        font: crate::text::BUILTIN_ICONS,
                        size: bounds.height * 0.7,
                        bounds: bounds,
                        color: style.checked_color,
                        horizontal_alignment: HorizontalAlignment::Center,
                        vertical_alignment: VerticalAlignment::Center,
                    };

                    vec![checkbox_border, checkbox_box, check, label]
                } else {
                    vec![checkbox_border, checkbox_box, label]
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
