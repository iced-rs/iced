//! Distribute content vertically.
use crate::layout;
use crate::renderer;
use crate::widget::Tree;
use crate::{Element, Layout, Length, Point, Rectangle, Size, Widget, Animation};

/// An amount of empty space.
///
/// It can be useful if you want to fill some space with nothing.
#[derive(Debug)]
pub struct Space {
    width: Animation,
    height: Animation,
}

impl Space {
    /// Creates an amount of empty [`Space`] with the given width and height.
    pub fn new(width: Length, height: Length) -> Self {
        Space { width: Animation::new_idle(width), height: Animation::new_idle(height) }
    }

    /// Creates an amount of horizontal [`Space`].
    pub fn with_width(width: Length) -> Self {
        Space {
            width: Animation::new_idle(width),
            height: Animation::new_idle(Length::Shrink),
        }
    }

    /// Creates an amount of vertical [`Space`].
    pub fn with_height(height: Length) -> Self {
        Space {
            width: Animation::new_idle(Length::Shrink),
            height: Animation::new_idle(height),
        }
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Space
where
    Renderer: crate::Renderer,
{
    fn width(&self) -> Animation {
        self.width
    }

    fn height(&self) -> Animation {
        self.height
    }

    fn layout(
        &self,
        _renderer: &Renderer,
        limits: &layout::Limits,
        tree: &Tree,
    ) -> layout::Node {
        let limits = limits.width(self.width.at()).height(self.height.at());

        layout::Node::new(limits.resolve(Size::ZERO))
    }

    fn draw(
        &self,
        _state: &Tree,
        _renderer: &mut Renderer,
        _theme: &Renderer::Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
    }
}

impl<'a, Message, Renderer> From<Space> for Element<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Message: 'a,
{
    fn from(space: Space) -> Element<'a, Message, Renderer> {
        Element::new(space)
    }
}
