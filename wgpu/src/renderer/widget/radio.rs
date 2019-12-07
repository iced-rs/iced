use crate::{Primitive, RadioStyle, Renderer};
use iced_native::{radio, MouseCursor, Rectangle};

const SIZE: f32 = 28.0;
const DOT_SIZE: f32 = SIZE / 2.0;

impl radio::Renderer for Renderer {
    type WidgetStyle = RadioStyle;

    fn default_size(&self) -> u32 {
        SIZE as u32
    }

    fn draw(
        &mut self,
        bounds: Rectangle,
        is_selected: bool,
        is_mouse_over: bool,
        style: &Self::WidgetStyle,
        (label, _): Self::Output,
    ) -> Self::Output {
        let (radio_border, radio_box) = (
            Primitive::Quad {
                bounds,
                background: if is_mouse_over {
                    style.get_border_hovered_color()
                } else {
                    style.border_color
                }
                    .into(),
                border_radius: (SIZE / 2.0) as u16,
            },
            Primitive::Quad {
                bounds: Rectangle {
                    x: bounds.x + style.border_width as f32,
                    y: bounds.y + style.border_width as f32,
                    width: bounds.width - (style.border_width * 2) as f32,
                    height: bounds.height - (style.border_width * 2) as f32,
                },
                background: style.background,
                border_radius: (SIZE / 2.0) as u16 - style.border_width,
            },
        );

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
                        background: style.dot_background,
                        border_radius: (DOT_SIZE / 2.0) as u16,
                    };

                    vec![radio_border, radio_box, radio_circle, label]
                } else {
                    vec![radio_border, radio_box, label]
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
