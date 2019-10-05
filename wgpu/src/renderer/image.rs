use crate::{Primitive, Renderer};
use iced_native::{image, Image, Layout, Node, Style};

impl image::Renderer<&str> for Renderer {
    fn node(&mut self, _image: &Image<&str>) -> Node {
        Node::new(Style::default())
    }

    fn draw(
        &mut self,
        _image: &Image<&str>,
        _layout: Layout<'_>,
    ) -> Self::Primitive {
        Primitive::None
    }
}
