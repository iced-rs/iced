use crate::{Primitive, Renderer};
use iced_native::{image, Image, Layout, MouseCursor, Node, Style};

impl image::Renderer<&str> for Renderer {
    fn node(&self, _image: &Image<&str>) -> Node {
        Node::new(Style::default())
    }

    fn draw(
        &mut self,
        _image: &Image<&str>,
        _layout: Layout<'_>,
    ) -> Self::Output {
        (Primitive::None, MouseCursor::OutOfBounds)
    }
}
