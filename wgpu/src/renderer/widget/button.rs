use crate::{Primitive, Renderer};
use iced_native::{
    button, Align, Background, Button, Layout, Length, MouseCursor,
    Node, Point, Rectangle, Style,
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

        let is_mouse_over = bounds.contains(cursor_position);

        // TODO: Render proper shadows
        // TODO: Make hovering and pressed styles configurable
        let shadow_offset = if is_mouse_over {
            if button.state.is_pressed {
                0.0
            } else {
                2.0
            }
        } else {
            1.0
        };

        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 1.0,
                            y: bounds.y + shadow_offset,
                            ..bounds
                        },
                        background: Background::Color(
                            [0.0, 0.0, 0.0, 0.5].into(),
                        ),
                        border_radius: button.border_radius,
                    },
                    Primitive::Quad {
                        bounds,
                        background: button.background.unwrap_or(
                            Background::Color([0.8, 0.8, 0.8].into()),
                        ),
                        border_radius: button.border_radius,
                    },
                    content,
                ],
            },
            if is_mouse_over {
                MouseCursor::Pointer
            } else {
                MouseCursor::OutOfBounds
            },
        )
    }
}
