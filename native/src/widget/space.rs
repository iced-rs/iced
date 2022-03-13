//! Distribute content vertically.
use crate::layout;
use crate::{Element, Layout, Length, Point, Rectangle, Size, Widget};

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
    pub fn new(width: Length, height: Length) -> Self {
        Space { width, height }
    }

    /// Creates an amount of horizontal [`Space`].
    pub fn with_width(width: Length) -> Self {
        Space {
            width,
            height: Length::Shrink,
        }
    }

    /// Creates an amount of vertical [`Space`].
    pub fn with_height(height: Length) -> Self {
        Space {
            width: Length::Shrink,
            height,
        }
    }
}

impl<Message, Renderer, Styling, Theme> Widget<Message, Renderer, Styling>
    for Space
where
    Styling: iced_style::Styling<Theme = Theme>,
    Renderer: crate::Renderer<Styling>,
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
        _renderer: &mut Renderer,
        _theme: &Theme,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
    }
}

impl<'a, Message, Renderer, Styling, Theme> From<Space>
    for Element<'a, Message, Renderer, Styling>
where
    Styling: iced_style::Styling<Theme = Theme>,
    Renderer: crate::Renderer<Styling>,
    Message: 'a,
{
    fn from(space: Space) -> Element<'a, Message, Renderer, Styling> {
        Element::new(space)
    }
}
