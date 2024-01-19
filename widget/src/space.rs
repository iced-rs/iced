//! Distribute content vertically.
use crate::core;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{Element, Layout, Length, Rectangle, Size, Widget};

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
    pub fn new(width: impl Into<Length>, height: impl Into<Length>) -> Self {
        Space {
            width: width.into(),
            height: height.into(),
        }
    }

    /// Creates an amount of horizontal [`Space`].
    pub fn with_width(width: impl Into<Length>) -> Self {
        Space {
            width: width.into(),
            height: Length::Shrink,
        }
    }

    /// Creates an amount of vertical [`Space`].
    pub fn with_height(height: impl Into<Length>) -> Self {
        Space {
            width: Length::Shrink,
            height: height.into(),
        }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Space
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
        &self,
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
        _theme: &Renderer::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }
}

impl<'a, Message, Renderer> From<Space> for Element<'a, Message, Renderer>
where
    Renderer: core::Renderer,
    Message: 'a,
{
    fn from(space: Space) -> Element<'a, Message, Renderer> {
        Element::new(space)
    }
}
