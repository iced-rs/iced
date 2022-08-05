//! Display interactive elements on top of other widgets.
mod element;

pub mod menu;

pub use element::Element;
pub use menu::Menu;

use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::widget::tree::{self, Tree};
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

    /// Returns the [`Tag`] of the [`Widget`].
    ///
    /// [`Tag`]: tree::Tag
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    /// Returns the [`State`] of the [`Widget`].
    ///
    /// [`State`]: tree::State
    fn state(&self) -> tree::State {
        tree::State::None
    }

    /// Returns the state [`Tree`] of the children of the [`Widget`].
    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    /// Reconciliates the [`Widget`] with the provided [`Tree`].
    fn diff(&self, _tree: &mut Tree) {}

    /// Applies an [`Operation`] to the [`Widget`].
    fn operate(
        &self,
        _layout: Layout<'_>,
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
}

/// Obtains the first overlay [`Element`] found in the given children.
///
/// This method will generally only be used by advanced users that are
/// implementing the [`Widget`](crate::Widget) trait.
pub fn from_children<'a, Message, Renderer>(
    children: &'a [crate::Element<'_, Message, Renderer>],
    tree: &'a mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
) -> Option<Element<'a, Message, Renderer>>
where
    Renderer: crate::Renderer,
{
    children
        .iter()
        .zip(&mut tree.children)
        .zip(layout.children())
        .filter_map(|((child, state), layout)| {
            child.as_widget().overlay(state, layout, renderer)
        })
        .next()
}
