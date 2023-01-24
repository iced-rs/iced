//! Display interactive elements on top of other widgets.
mod element;
mod group;

pub mod menu;

pub use element::Element;
pub use group::Group;
pub use menu::Menu;

use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::widget::Tree;
use crate::{Clipboard, Layout, Point, Rectangle, Shell, Size};

/// An interactive component that can be displayed on top of other widgets.
pub trait Overlay<Message, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Returns the layout [`Node`] of the [`Overlay`].
    ///
    /// This [`Node`] is used by the runtime to compute the [`Layout`] of the
    /// user interface.
    ///
    /// [`Node`]: layout::Node
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node;

    /// Draws the [`Overlay`] using the associated `Renderer`.
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    );

    /// Applies a [`widget::Operation`] to the [`Overlay`].
    fn operate(
        &mut self,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn widget::Operation<Message>,
    ) {
    }

    /// Processes a runtime [`Event`].
    ///
    /// It receives:
    ///   * an [`Event`] describing user interaction
    ///   * the computed [`Layout`] of the [`Overlay`]
    ///   * the current cursor position
    ///   * a mutable `Message` list, allowing the [`Overlay`] to produce
    ///   new messages based on user interaction.
    ///   * the `Renderer`
    ///   * a [`Clipboard`], if available
    ///
    /// By default, it does nothing.
    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        event::Status::Ignored
    }

    /// Returns the current [`mouse::Interaction`] of the [`Overlay`].
    ///
    /// By default, it returns [`mouse::Interaction::Idle`].
    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::Idle
    }

    /// Returns true if the cursor is over the [`Overlay`].
    ///
    /// By default, it returns true if the bounds of the `layout` contain
    /// the `cursor_position`.
    fn is_over(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        layout.bounds().contains(cursor_position)
    }
}

/// Returns a [`Group`] of overlay [`Element`] children.
///
/// This method will generally only be used by advanced users that are
/// implementing the [`Widget`](crate::Widget) trait.
pub fn from_children<'a, Message, Renderer>(
    children: &'a mut [crate::Element<'_, Message, Renderer>],
    tree: &'a mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> Option<Element<'a, Message, Renderer>>
where
    Renderer: crate::Renderer,
{
    let children = children
        .iter_mut()
        .zip(&mut tree.children)
        .zip(layout.children())
        .filter_map(|((child, state), layout)| {
            child.as_widget_mut().overlay(state, layout, renderer)
        })
        .collect::<Vec<_>>();

    (!children.is_empty()).then(|| Group::with_children(children).overlay())
}
