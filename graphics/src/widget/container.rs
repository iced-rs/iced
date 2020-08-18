//! Decorate content and apply alignment.
use crate::container;
use crate::defaults::{self, Defaults};
use crate::{Backend, Primitive, Renderer};
use iced_native::{Background, Color, Element, Layout, Point, Rectangle};

pub use iced_style::container::{Style, StyleSheet};

/// An element decorating some content.
///
/// This is an alias of an `iced_native` container with a default
/// `Renderer`.
pub type Container<'a, Message, Backend> =
    iced_native::Container<'a, Message, Renderer<Backend>>;

impl<B> iced_native::container::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn container::StyleSheet>;

    fn draw<Message>(
        &mut self,
        defaults: &Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        style_sheet: &Self::Style,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> Self::Output {
        let style = style_sheet.style();

        let defaults = Defaults {
            text: defaults::Text {
                color: style.text_color.unwrap_or(defaults.text.color),
            },
        };

        let (content, mouse_interaction) = content.draw(
            self,
            &defaults,
            content_layout,
            cursor_position,
            viewport,
        );

        if let Some(background) = background(bounds, &style) {
            (
                Primitive::Group {
                    primitives: vec![background, content],
                },
                mouse_interaction,
            )
        } else {
            (content, mouse_interaction)
        }
    }
}

pub(crate) fn background(
    bounds: Rectangle,
    style: &container::Style,
) -> Option<Primitive> {
    if style.background.is_some() || style.border_width > 0 {
        Some(Primitive::Quad {
            bounds,
            background: style
                .background
                .unwrap_or(Background::Color(Color::TRANSPARENT)),
            border_radius: style.border_radius,
            border_width: style.border_width,
            border_color: style.border_color,
        })
    } else {
        None
    }
}
