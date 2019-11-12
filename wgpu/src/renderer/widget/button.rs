use crate::{Primitive, Renderer};
use iced_native::{
    button, layout, Background, Button, Layout, Length, MouseCursor, Point,
    Rectangle,
};

impl button::Renderer for Renderer {
    fn layout<Message>(
        &self,
        button: &Button<Message, Self>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let padding = f32::from(button.padding);
        let limits = limits
            .min_width(100)
            .width(button.width)
            .height(Length::Shrink)
            .pad(padding);

        let mut content = button.content.layout(self, &limits);

        content.bounds.x = padding;
        content.bounds.y = padding;

        let size = limits.resolve(content.size()).pad(padding);

        layout::Node::with_children(size, vec![content])
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
