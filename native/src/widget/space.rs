//! Distribute content vertically.
use std::hash::Hash;

use crate::{
    layout, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

/// An amount of empty space.
///
/// It can be useful if you want to fill some space with nothing.
#[derive(Debug)]
pub struct Space {
    width: Length,
    height: Length,
}

impl Space {
    /// Creates an amount of empty [`Space`] with the given width and height.
    ///
    /// [`Space`]: struct.Space.html
    pub fn new(width: Length, height: Length) -> Self {
        Space { width, height }
    }

    /// Creates an amount of horizontal [`Space`].
    ///
    /// [`Space`]: struct.Space.html
    pub fn with_width(width: Length) -> Self {
        Space {
            width,
            height: Length::Shrink,
        }
    }

    /// Creates an amount of vertical [`Space`].
    ///
    /// [`Space`]: struct.Space.html
    pub fn with_height(height: Length) -> Self {
        Space {
            width: Length::Shrink,
            height,
        }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Space
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
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(layout.bounds())
    }

    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Space>().hash(state);

        self.width.hash(state);
        self.height.hash(state);
    }
}

/// The renderer of an amount of [`Space`].
///
/// [`Space`]: struct.Space.html
pub trait Renderer: crate::Renderer {
    /// Draws an amount of empty [`Space`].
    ///
    /// You should most likely return an empty primitive here.
    ///
    /// [`Space`]: struct.Space.html
    fn draw(&mut self, bounds: Rectangle) -> Self::Output;
}

impl<'a, Message, Renderer> From<Space> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'a,
{
    fn from(space: Space) -> Element<'a, Message, Renderer> {
        Element::new(space)
    }
}
