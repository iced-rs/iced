use crate::{Primitive, Renderer};
use iced_native::{slider, Layout, MouseCursor, Node, Point, Slider, Style};

impl slider::Renderer for Renderer {
    fn node<Message>(&self, _slider: &Slider<Message>) -> Node {
        Node::new(Style::default())
    }

    fn draw<Message>(
        &mut self,
        _slider: &Slider<Message>,
        _layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Self::Output {
        (Primitive::None, MouseCursor::OutOfBounds)
    }
}
