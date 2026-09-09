//! Dummy widgets for testing.
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::{Tree, Widget};
use crate::core::{Element, Length, Rectangle, Size, Vector};
use crate::overlay::with_index;

/// A widget that produces two overlays.
struct TwoOverlays;

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for TwoOverlays
where
    Renderer: crate::core::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        _limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(Size::new(100.0, 100.0))
    }

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut Tree,
        _layout: Layout<'a>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        vec![with_index(1.0), with_index(2.0)]
    }
}

/// Returns an [`Element`] wrapping a dummy widget that produces two
/// overlays.
pub fn two_overlays<Message, Theme, Renderer>() -> Element<'static, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    Element::new(TwoOverlays)
}
