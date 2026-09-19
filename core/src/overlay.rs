//! Display interactive elements on top of other widgets.
mod element;
mod group;

pub use element::Element;
pub use group::Group;

use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::widget::Tree;
use crate::{Event, Layout, Rectangle, Shell, Size, Vector};

/// An interactive component that can be displayed on top of other widgets.
pub trait Overlay<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Draws the [`Overlay`] using the associated `Renderer`.
    ///
    /// Implementors are expected to call
    /// [`Renderer::with_layer`](renderer::Renderer::with_layer) themselves,
    /// so that their contents are properly clipped.
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        cursor: mouse::Cursor,
    );

    /// Applies a [`widget::Operation`] to the [`Overlay`].
    fn operate(&mut self, _renderer: &Renderer, _operation: &mut dyn widget::Operation) {}

    /// Processes a runtime [`Event`].
    ///
    /// By default, it does nothing.
    fn update(
        &mut self,
        _event: &Event,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _shell: &mut Shell<'_, Message>,
    ) {
    }

    /// Returns the current [`mouse::Interaction`] of the [`Overlay`].
    ///
    /// By default, it returns [`mouse::Interaction::None`].
    fn mouse_interaction(
        &self,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }

    /// Returns the nested overlays of the [`Overlay`].
    fn overlay<'a>(
        &'a mut self,
        _renderer: &Renderer,
    ) -> Vec<Element<'a, Message, Theme, Renderer>> {
        Vec::new()
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

/// Returns the overlays of the given [`Element`] children.
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
    window: Size,
) -> Vec<Element<'a, Message, Theme, Renderer>>
where
    Renderer: crate::Renderer,
{
    children
        .iter_mut()
        .zip(&mut tree.children)
        .zip(layout.children())
        .flat_map(|((child, state), layout)| {
            child
                .as_widget_mut()
                .overlay(state, layout, renderer, viewport, translation, window)
        })
        .collect()
}
