use crate::{radio::StyleSheet, Primitive, Renderer};
use iced_native::{mouse, radio, Background, Color, Rectangle};

const SIZE: f32 = 28.0;
const DOT_SIZE: f32 = SIZE / 2.0;

impl radio::Renderer for Renderer {
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_SIZE: u16 = SIZE as u16;
    const DEFAULT_SPACING: u16 = 15;

    fn draw(
        &mut self,
        bounds: Rectangle,
        is_selected: bool,
        is_mouse_over: bool,
        (label, _): Self::Output,
        style_sheet: &Self::Style,
    ) -> Self::Output {
        let style = if is_mouse_over {
            style_sheet.hovered()
        } else {
            style_sheet.active()
        };

        let radio = Primitive::Quad {
            bounds,
            background: style.background,
            border_radius: (SIZE / 2.0) as u16,
            border_width: style.border_width,
            border_color: style.border_color,
        };

        (
            Primitive::Group {
                primitives: if is_selected {
                    let radio_circle = Primitive::Quad {
                        bounds: Rectangle {
                            x: bounds.x + DOT_SIZE / 2.0,
                            y: bounds.y + DOT_SIZE / 2.0,
                            width: bounds.width - DOT_SIZE,
                            height: bounds.height - DOT_SIZE,
                        },
                        background: Background::Color(style.dot_color),
                        border_radius: (DOT_SIZE / 2.0) as u16,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    };

                    vec![radio, radio_circle, label]
                } else {
                    vec![radio, label]
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
