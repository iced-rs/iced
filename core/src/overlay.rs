//! Display interactive elements on top of other widgets.
mod element;
mod group;

pub use element::Element;
pub use group::Group;

use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::widget::Tree;
use crate::{Clipboard, Event, Layout, Rectangle, Shell, Size, Vector};

/// An interactive component that can be displayed on top of other widgets.
pub trait Overlay<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Returns the layout [`Node`] of the [`Overlay`].
    ///
    /// This [`Node`] is used by the runtime to compute the [`Layout`] of the
    /// user interface.
    ///
    /// [`Node`]: layout::Node
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node;

    /// Draws the [`Overlay`] using the associated `Renderer`.
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    );

    /// Applies a [`widget::Operation`] to the [`Overlay`].
    fn operate(
        &mut self,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _operation: &mut dyn widget::Operation,
    ) {
    }

    /// Processes a runtime [`Event`].
    ///
    /// It receives:
    ///   * an [`Event`] describing user interaction
    ///   * the computed [`Layout`] of the [`Overlay`]
    ///   * the current cursor position
    ///   * a mutable `Message` list, allowing the [`Overlay`] to produce
    ///     new messages based on user interaction.
    ///   * the `Renderer`
    ///   * a [`Clipboard`], if available
    ///
    /// By default, it does nothing.
    fn update(
        &mut self,
        _event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _shell: &mut Shell<'_, Message>,
    ) {
    }

    /// Returns the current [`mouse::Interaction`] of the [`Overlay`].
    ///
    /// By default, it returns [`mouse::Interaction::None`].
    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }

    /// Returns the nested overlay of the [`Overlay`], if there is any.
    fn overlay<'a>(
        &'a mut self,
        _layout: Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<Element<'a, Message, Theme, Renderer>> {
        None
    }

    /// The index of the overlay.
    ///
    /// Overlays with a higher index will be rendered on top of overlays with
    /// a lower index.
    ///
    /// By default, it returns `1.0`.
    fn index(&self) -> f32 {
        1.0
    }
}

/// Returns a [`Group`] of overlay [`Element`] children.
///
/// This method will generally only be used by advanced users that are
/// implementing the [`Widget`](crate::Widget) trait.
pub fn from_children<'a, Message, Theme, Renderer>(
    children: &'a mut [crate::Element<'_, Message, Theme, Renderer>],
    tree: &'a mut Tree,
    layout: Layout<'a>,
    renderer: &Renderer,
    viewport: &Rectangle,
    translation: Vector,
) -> Option<Element<'a, Message, Theme, Renderer>>
where
    Renderer: crate::Renderer,
{
    let children = children
        .iter_mut()
        .zip(&mut tree.children)
        .zip(layout.children())
        .filter_map(|((child, state), layout)| {
            child.as_widget_mut().overlay(
                state,
                layout,
                renderer,
                viewport,
                translation,
            )
        })
        .collect::<Vec<_>>();

    (!children.is_empty()).then(|| Group::with_children(children).overlay())
}
