use crate::{Background, Primitive, Renderer};
use iced_native::{button, Button, Color, Layout, Length, Node, Point, Style};

impl button::Renderer for Renderer {
    fn node<Message>(&self, button: &Button<Message>) -> Node {
        let style = Style::default()
            .width(button.width)
            .min_height(Length::Units(30))
            .min_width(Length::Units(100))
            .align_self(button.align_self);

        Node::new(style)
    }

    fn draw<Message>(
        &mut self,
        button: &Button<Message>,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        Primitive::Group {
            primitives: vec![
                Primitive::Box {
                    bounds: layout.bounds(),
                    background: Background::Color(Color {
                        r: 0.0,
                        b: 1.0,
                        g: 0.0,
                        a: 1.0,
                    }),
                },
                Primitive::Text {
                    content: button.label.clone(),
                    size: 20.0,
                    bounds: layout.bounds(),
                },
            ],
        }
    }
}
