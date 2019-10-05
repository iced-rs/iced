use crate::{Primitive, Renderer};
use iced_native::{button, Button, Layout, Node, Point, Style};

impl button::Renderer for Renderer {
    fn node<Message>(&self, _button: &Button<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _button: &Button<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        // TODO
        Primitive::None
    }
}
