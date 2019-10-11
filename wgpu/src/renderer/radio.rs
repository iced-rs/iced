use crate::{Primitive, Renderer};
use iced_native::{radio, Layout, Node, Point, Radio, Style};

impl radio::Renderer for Renderer {
    fn node<Message>(&self, _checkbox: &Radio<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _radio: &Radio<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Output {
        Primitive::None
    }
}
