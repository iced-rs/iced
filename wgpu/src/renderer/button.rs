use crate::{Primitive, Renderer};
use iced_native::{
    button, Align, Background, Button, Color, Layout, Length, MouseCursor,
    Node, Point, Style,
};

impl button::Renderer for Renderer {
    fn node<Message>(&self, button: &Button<Message, Self>) -> Node {
        let style = Style::default()
            .width(button.width)
            .padding(button.padding)
            .min_width(Length::Units(100))
            .align_self(button.align_self)
            .align_items(Align::Stretch);

        Node::with_children(style, vec![button.content.node(self)])
    }

    fn draw<Message>(
        &mut self,
        button: &Button<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Self::Output {
        let bounds = layout.bounds();

        let (content, _) = button.content.draw(
            self,
            layout.children().next().unwrap(),
            cursor_position,
        );

        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds,
                        background: button.background.unwrap_or(
                            Background::Color(Color {
                                r: 0.8,
                                b: 0.8,
                                g: 0.8,
                                a: 1.0,
                            }),
                        ),
                        border_radius: button.border_radius,
                    },
                    content,
                ],
            },
            if bounds.contains(cursor_position) {
                MouseCursor::Pointer
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
