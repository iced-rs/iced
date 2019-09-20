//! Write some text for your users to read.
use crate::{Element, Hasher, Layout, MouseCursor, Node, Point, Widget};

use std::hash::Hash;

pub use iced_core::text::*;

impl<Message, Renderer> Widget<Message, Renderer> for Text
where
    Renderer: self::Renderer,
{
    fn node(&self, renderer: &mut Renderer) -> Node {
        renderer.node(&self)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(&self, layout);

        MouseCursor::OutOfBounds
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.content.hash(state);
        self.size.hash(state);
    }
}

/// The renderer of a [`Text`] fragment.
///
/// Your [renderer] will need to implement this trait before being
/// able to use [`Text`] in your [`UserInterface`].
///
/// [`Text`]: struct.Text.html
/// [renderer]: ../../renderer/index.html
/// [`UserInterface`]: ../../struct.UserInterface.html
pub trait Renderer {
    /// Creates a [`Node`] with the given [`Style`] for the provided [`Text`]
    /// contents and size.
    ///
    /// You should probably use [`Node::with_measure`] to allow [`Text`] to
    /// adapt to the dimensions of its container.
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Style`]: ../../struct.Style.html
    /// [`Text`]: struct.Text.html
    /// [`Node::with_measure`]: ../../struct.Node.html#method.with_measure
    fn node(&self, text: &Text) -> Node;

    /// Draws a [`Text`] fragment.
    ///
    /// It receives:
    ///   * the bounds of the [`Text`]
    ///   * the contents of the [`Text`]
    ///   * the size of the [`Text`]
    ///   * the color of the [`Text`]
    ///   * the [`HorizontalAlignment`] of the [`Text`]
    ///   * the [`VerticalAlignment`] of the [`Text`]
    ///
    /// [`Text`]: struct.Text.html
    /// [`HorizontalAlignment`]: enum.HorizontalAlignment.html
    /// [`VerticalAlignment`]: enum.VerticalAlignment.html
    fn draw(&mut self, text: &Text, layout: Layout<'_>);
}

impl<'a, Message, Renderer> From<Text> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(text: Text) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}
