//! Display images in your user interface.

use crate::{Element, Hasher, Layout, Node, Point, Widget};

use std::hash::Hash;

pub use iced_core::Image;

impl<I, Message, Renderer> Widget<Message, Renderer> for Image<I>
where
    Renderer: self::Renderer<I>,
    I: Clone,
{
    fn node(&self, renderer: &Renderer) -> Node {
        renderer.node(&self)
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
        self.align_self.hash(state);
    }
}

/// The renderer of an [`Image`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// an [`Image`] in your user interface.
///
/// [`Image`]: struct.Image.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer<I>: crate::Renderer {
    /// Creates a [`Node`] for the provided [`Image`].
    ///
    /// You should probably keep the original aspect ratio, if possible.
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Image`]: struct.Image.html
    fn node(&self, image: &Image<I>) -> Node;

    /// Draws an [`Image`].
    ///
    /// [`Image`]: struct.Image.html
    fn draw(&mut self, image: &Image<I>, layout: Layout<'_>) -> Self::Output;
}

impl<'a, I, Message, Renderer> From<Image<I>> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer<I>,
    I: Clone + 'a,
{
    fn from(image: Image<I>) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
