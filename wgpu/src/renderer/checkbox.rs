use crate::{Primitive, Renderer};
use iced_native::{checkbox, Checkbox, Layout, Node, Point, Style};

impl checkbox::Renderer for Renderer {
    fn node<Message>(&self, _checkbox: &Checkbox<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _checkbox: &Checkbox<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Primitive {
        // TODO
        Primitive::None
    }
}
