//! Assign explicit focus traversal priority to a widget.
//!
//! Wrapping a focusable widget with [`focus_order`] tells the spatial
//! navigation algorithm to prefer lower-order widgets when two candidates
//! are otherwise equidistant. This is useful when the default spatial
//! heuristic picks the wrong widget (e.g. a wide button wins over a
//! nearby toggle due to more beam overlap).
//!
//! ```no_run
//! use iced::widget::{focus_order, toggler};
//! use iced::Element;
//!
//! fn view(value: bool) -> Element<'static, bool> {
//!     // Toggle gets focus before siblings with higher / default order
//!     focus_order(toggler(value).on_toggle(|v| v), 0).into()
//! }
//! ```
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{Element, Event, Length, Rectangle, Shell, Size, Vector, Widget};

/// A transparent wrapper that sets a focus-order hint on its content.
///
/// Lower values receive focus first when two candidates are at the same
/// spatial distance. Widgets without a [`FocusOrder`] wrapper default to
/// `u32::MAX` (lowest priority).
#[allow(missing_debug_implementations)]
pub struct FocusOrder<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    order: u32,
}

impl<'a, Message, Theme, Renderer> FocusOrder<'a, Message, Theme, Renderer> {
    /// Creates a new [`FocusOrder`] wrapper with the given priority.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>, order: u32) -> Self {
        Self {
            content: content.into(),
            order,
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for FocusOrder<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.focus_order_hint(self.order);
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<FocusOrder<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(focus_order: FocusOrder<'a, Message, Theme, Renderer>) -> Self {
        Self::new(focus_order)
    }
}

/// Creates a [`FocusOrder`] wrapper that assigns explicit focus traversal
/// priority to its content.
///
/// Lower `order` values receive focus first when multiple candidates are
/// at the same spatial distance. Widgets without this wrapper default to
/// `u32::MAX`.
///
/// ```no_run
/// use iced::widget::{focus_order, toggler};
/// use iced::Element;
///
/// fn view(value: bool) -> Element<'static, bool> {
///     focus_order(toggler(value).on_toggle(|v| v), 0).into()
/// }
/// ```
pub fn focus_order<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
    order: u32,
) -> FocusOrder<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    FocusOrder::new(content, order)
}
