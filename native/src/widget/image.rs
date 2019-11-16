//! Display images in your user interface.

use crate::{layout, Element, Hasher, Layout, Length, Point, Widget};

use std::hash::Hash;

pub use iced_core::Image;

impl<Message, Renderer> Widget<Message, Renderer> for Image
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        renderer.layout(&self, limits)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(&self, layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
        self.height.hash(state);
    }
}

/// The renderer of an [`Image`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// an [`Image`] in your user interface.
///
/// [`Image`]: struct.Image.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Creates a [`Node`] for the provided [`Image`].
    ///
    /// You should probably keep the original aspect ratio, if possible.
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Image`]: struct.Image.html
    fn layout(&self, image: &Image, limits: &layout::Limits) -> layout::Node;

    /// Draws an [`Image`].
    ///
    /// [`Image`]: struct.Image.html
    fn draw(&mut self, image: &Image, layout: Layout<'_>) -> Self::Output;
}

impl<'a, Message, Renderer> From<Image> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(image: Image) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
