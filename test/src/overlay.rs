//! Dummy overlays for testing.
use crate::core::Size;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay::{self, Overlay};
use crate::core::renderer;

/// An overlay with a fixed index.
struct WithIndex(f32);

impl<Message, Theme, Renderer> Overlay<Message, Theme, Renderer> for WithIndex
where
    Renderer: crate::core::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: Size) -> layout::Node {
        layout::Node::new(Size::ZERO)
    }

    fn draw(
        &self,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
    }

    fn index(&self) -> f32 {
        self.0
    }
}

/// Returns an [`overlay::Element`] wrapping a dummy overlay with a fixed
/// index.
pub fn with_index<Message, Theme, Renderer>(
    index: f32,
) -> overlay::Element<'static, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    overlay::Element::new(Box::new(WithIndex(index)))
}
