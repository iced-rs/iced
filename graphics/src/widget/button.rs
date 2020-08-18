//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: type.Button.html
//! [`State`]: struct.State.html
use crate::defaults::{self, Defaults};
use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::{
    Background, Color, Element, Layout, Point, Rectangle, Vector,
};

pub use iced_native::button::State;
pub use iced_style::button::{Style, StyleSheet};

/// A widget that produces a message when clicked.
///
/// This is an alias of an `iced_native` button with an `iced_wgpu::Renderer`.
pub type Button<'a, Message, Backend> =
    iced_native::Button<'a, Message, Renderer<Backend>>;

impl<B> iced_native::button::Renderer for Renderer<B>
where
    B: Backend,
{
    const DEFAULT_PADDING: u16 = 5;

    type Style = Box<dyn StyleSheet>;

    fn draw<Message>(
        &mut self,
        _defaults: &Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        is_disabled: bool,
        is_pressed: bool,
        style: &Box<dyn StyleSheet>,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let styling = if is_disabled {
            style.disabled()
        } else if is_mouse_over {
            if is_pressed {
                style.pressed()
            } else {
                style.hovered()
            }
        } else {
            style.active()
        };

        let (content, _) = content.draw(
            self,
            &Defaults {
                text: defaults::Text {
                    color: styling.text_color,
                },
            },
            content_layout,
            cursor_position,
            &bounds,
        );

        (
            if styling.background.is_some() || styling.border_width > 0 {
                let background = Primitive::Quad {
                    bounds,
                    background: styling
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                    border_radius: styling.border_radius,
                    border_width: styling.border_width,
                    border_color: styling.border_color,
                };

                if styling.shadow_offset == Vector::default() {
                    Primitive::Group {
                        primitives: vec![background, content],
                    }
                } else {
                    // TODO: Implement proper shadow support
                    let shadow = Primitive::Quad {
                        bounds: Rectangle {
                            x: bounds.x + styling.shadow_offset.x,
                            y: bounds.y + styling.shadow_offset.y,
                            ..bounds
                        },
                        background: Background::Color(
                            [0.0, 0.0, 0.0, 0.5].into(),
                        ),
                        border_radius: styling.border_radius,
                        border_width: 0,
                        border_color: Color::TRANSPARENT,
                    };

                    Primitive::Group {
                        primitives: vec![shadow, background, content],
                    }
                }
            } else {
                content
            },
            if is_mouse_over && !is_disabled {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
