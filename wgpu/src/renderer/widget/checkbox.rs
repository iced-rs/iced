use crate::{Primitive, Renderer};
use iced_native::{
    checkbox, Background, HorizontalAlignment, MouseCursor, Rectangle,
    VerticalAlignment,
};

const SIZE: f32 = 28.0;

impl checkbox::Renderer for Renderer {
    fn default_size(&self) -> u32 {
        SIZE as u32
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        is_checked: bool,
        is_mouse_over: bool,
        (label, _): Self::Output,
    ) -> Self::Output {
        let checkbox_box = Primitive::Quad {
            bounds,
            background: Background::Color(
                if is_mouse_over {
                    [0.90, 0.90, 0.90]
                } else {
                    [0.95, 0.95, 0.95]
                }
                .into(),
            ),
            border_radius: 5,
            border_color: [0.6, 0.6, 0.6].into(),
            border_width: 1,
        };

        (
            Primitive::Group {
                primitives: if is_checked {
                    let check = Primitive::Text {
                        content: crate::text::CHECKMARK_ICON.to_string(),
                        font: crate::text::BUILTIN_ICONS,
                        size: bounds.height * 0.7,
                        bounds,
                        color: [0.3, 0.3, 0.3].into(),
                        horizontal_alignment: HorizontalAlignment::Center,
                        vertical_alignment: VerticalAlignment::Center,
                    };

                    vec![checkbox_box, check, label]
                } else {
                    vec![checkbox_box, label]
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
