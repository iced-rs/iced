//! Add some explicit spacing between elements.
use crate::core;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{Element, Layout, Length, Rectangle, Size, Widget};

/// Creates a new [`Space`] widget that fills the available
/// horizontal space.
///
/// This can be useful to separate widgets in a [`Row`](crate::Row).
pub fn horizontal() -> Space {
    Space::new().width(Length::Fill)
}

/// Creates a new [`Space`] widget that fills the available
/// vertical space.
///
/// This can be useful to separate widgets in a [`Column`](crate::Column).
pub fn vertical() -> Space {
    Space::new().height(Length::Fill)
}

/// An amount of empty space.
///
/// It can be useful if you want to fill some space with nothing.
#[derive(Debug)]
pub struct Space {
    width: Length,
    height: Length,
}

impl Space {
    /// Creates some empty [`Space`] with no size.
    pub fn new() -> Self {
        Space {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the width of the [`Space`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Space`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl Default for Space {
    fn default() -> Self {
        Space::new()
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Space
where
    Renderer: core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _state: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}

impl<'a, Message, Theme, Renderer> From<Space>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
    Message: 'a,
{
    fn from(space: Space) -> Element<'a, Message, Theme, Renderer> {
        Element::new(space)
    }
}
