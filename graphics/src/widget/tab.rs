//! Create choices using tab buttons.
use crate::defaults::{self, Defaults};
use crate::{Backend, Primitive, Renderer};
use iced_native::{mouse, tab};
use iced_native::{Background, Color, Element, Layout, Point, Rectangle};

pub use iced_style::tab::{
    Indicator, Position, Style, StyleDefaultVertical, StyleSheet,
};

/// Create choices for using tab buttons.
pub type Tab<'a, Message, Backend> =
    iced_native::Tab<'a, Message, Renderer<Backend>>;

impl<B> tab::Renderer for Renderer<B>
where
    B: Backend,
{
    const DEFAULT_PADDING: u16 = 5;

    type Style = Box<dyn StyleSheet>;

    fn draw<Message>(
        &mut self,
        _defaults: &Self::Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        is_selected: bool,
        style_sheet: &Self::Style,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> Self::Output {
        let is_mouse_over = bounds.contains(cursor_position);

        let style = if is_mouse_over {
            if is_selected {
                style_sheet.selected_hovered()
            } else {
                style_sheet.unselected_hovered()
            }
        } else {
            if is_selected {
                style_sheet.selected()
            } else {
                style_sheet.unselected()
            }
        };

        let (content, _) = content.draw(
            self,
            &Defaults {
                text: defaults::Text {
                    color: style.text_color,
                },
            },
            content_layout,
            cursor_position,
            &bounds,
        );

        let indicator = if let Some(indicator_style) = style.indicator {
            if indicator_style.position == Position::Bottom
                || indicator_style.position == Position::Top
            {
                let (x, width) = if let Some(length) = indicator_style.length {
                    (
                        bounds.x + ((bounds.width - f32::from(length)) / 2.0),
                        f32::from(length),
                    )
                } else {
                    (bounds.x, bounds.width)
                };

                let y = if indicator_style.position == Position::Bottom {
                    bounds.y + bounds.height
                        - indicator_style.thickness
                        - f32::from(indicator_style.offset)
                } else {
                    bounds.y + f32::from(indicator_style.offset)
                };

                Primitive::Quad {
                    bounds: Rectangle {
                        x,
                        y,
                        width,
                        height: indicator_style.thickness,
                    },
                    background: Background::Color(indicator_style.color),
                    border_radius: indicator_style.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            } else {
                let (y, height) = if let Some(length) = indicator_style.length {
                    (
                        bounds.y + ((bounds.height - f32::from(length)) / 2.0),
                        f32::from(length),
                    )
                } else {
                    (bounds.y, bounds.height)
                };

                let x = if indicator_style.position == Position::Right {
                    bounds.x + bounds.width
                        - indicator_style.thickness
                        - f32::from(indicator_style.offset)
                } else {
                    bounds.x + f32::from(indicator_style.offset)
                };

                Primitive::Quad {
                    bounds: Rectangle {
                        x,
                        y,
                        width: indicator_style.thickness,
                        height,
                    },
                    background: Background::Color(indicator_style.color),
                    border_radius: indicator_style.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }
            }
        } else {
            Primitive::None
        };

        (
            if style.background.is_some() || style.border_width > 0.0 {
                let background = Primitive::Quad {
                    bounds,
                    background: style
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                    border_radius: style.border_radius,
                    border_width: style.border_width,
                    border_color: style.border_color,
                };

                Primitive::Group {
                    primitives: vec![background, indicator, content],
                }
            } else {
                Primitive::Group {
                    primitives: vec![indicator, content],
                }
            },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}
