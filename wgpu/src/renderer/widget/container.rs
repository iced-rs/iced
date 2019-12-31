use crate::{container, defaults, Defaults, Primitive, Renderer};
use iced_native::{Element, Layout, Point, Rectangle};

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
            ..*defaults
        };

        let (content, mouse_cursor) =
            content.draw(self, &defaults, content_layout, cursor_position);

        match style.background {
            Some(background) => {
                let quad = Primitive::Quad {
                    bounds,
                    background,
                    border_radius: style.border_radius,
                };

                (
                    Primitive::Group {
                        primitives: vec![quad, content],
                    },
                    mouse_cursor,
                )
            }
            None => (content, mouse_cursor),
        }
    }
}
