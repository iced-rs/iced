use crate::{container, defaults, Defaults, Primitive, Renderer};
use iced_native::{Background, Color, Element, Layout, Point, Rectangle};

impl iced_native::container::Renderer for Renderer {
    type Style = Box<dyn container::StyleSheet>;

    fn draw<Message>(
        &mut self,
        defaults: &Defaults,
        bounds: Rectangle,
        cursor_position: Point,
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

        let (content, mouse_interaction) =
            content.draw(self, &defaults, content_layout, cursor_position);

        if style.background.is_some() || style.border_width > 0 {
            let quad = Primitive::Quad {
                bounds,
                background: style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
                border_radius: style.border_radius,
                border_width: style.border_width,
                border_color: style.border_color,
            };

            (
                Primitive::Group {
                    primitives: vec![quad, content],
                },
                mouse_interaction,
            )
        } else {
            (content, mouse_interaction)
        }
    }
}
