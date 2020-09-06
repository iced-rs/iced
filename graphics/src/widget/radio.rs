//! Create choices using radio buttons.
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::radio;
use iced_native::{Background, Color, Rectangle};

pub use iced_style::radio::{Style, StyleSheet};

/// A circular button representing a choice.
///
/// This is an alias of an `iced_native` radio button with an
/// `iced_wgpu::Renderer`.
pub type Radio<Message, Backend> =
    iced_native::Radio<Message, Renderer<Backend>>;

impl<B> radio::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_SIZE: u16 = 28;
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

        let size = bounds.width;
        let dot_size = size / 2.0;

        let radio = Primitive::Quad {
            bounds,
            background: style.background,
            border_radius: (size / 2.0) as u16,
            border_width: style.border_width,
            border_color: style.border_color,
        };

        (
            Primitive::Group {
                primitives: if is_selected {
                    let radio_circle = Primitive::Quad {
                        bounds: Rectangle {
                            x: bounds.x + dot_size / 2.0,
                            y: bounds.y + dot_size / 2.0,
                            width: bounds.width - dot_size,
                            height: bounds.height - dot_size,
                        },
                        background: Background::Color(style.dot_color),
                        border_radius: (dot_size / 2.0) as u16,
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
