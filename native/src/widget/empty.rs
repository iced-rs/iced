//! Distribute content vertically.
use std::hash::Hash;

use crate::{
    layout, Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget,
};

/// An amount of empty space.
///
/// It can be useful if you want to fill some space with nothing.
///
/// [`Empty`]: struct.Empty.html
#[derive(Debug)]
pub struct Empty {
    width: Length,
    height: Length,
}

impl Empty {
    /// Creates an amount of [`Empty`] space.
    ///
    /// [`Empty`]: struct.Empty.html
    pub fn new() -> Self {
        Empty {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the width of the [`Empty`] space.
    ///
    /// [`Empty`]: struct..html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Empty`] space.
    ///
    /// [`Empty`]: struct.Column.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Empty
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
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(layout.bounds())
    }

    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Empty>().hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

/// The renderer of an amount of [`Empty`] space.
///
/// [`Empty`]: struct.Empty.html
pub trait Renderer: crate::Renderer {
    /// Draws an amount of [`Empty`] space.
    ///
    /// You should most likely return an empty primitive here.
    fn draw(&mut self, bounds: Rectangle) -> Self::Output;
}

impl<'a, Message, Renderer> From<Empty> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: 'static,
{
    fn from(empty: Empty) -> Element<'a, Message, Renderer> {
        Element::new(empty)
    }
}
