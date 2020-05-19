use crate::{button::StyleSheet, defaults, Defaults, Primitive, Renderer};
use iced_native::{
    mouse, Background, Color, Element, Layout, Point, Rectangle, Vector,
};

impl iced_native::button::Renderer for Renderer {
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
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
