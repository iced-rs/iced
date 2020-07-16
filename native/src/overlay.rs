//! Display interactive elements on top of other widgets.
mod element;

pub mod menu;

pub use element::Element;
pub use menu::Menu;

use crate::{layout, Clipboard, Event, Hasher, Layout, Point, Size};

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
    /// [`Node`]: ../layout/struct.Node.html
    /// [`Widget`]: trait.Overlay.html
    /// [`Layout`]: ../layout/struct.Layout.html
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node;

    /// Draws the [`Overlay`] using the associated `Renderer`.
    ///
    /// [`Overlay`]: trait.Overlay.html
    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output;

    /// Computes the _layout_ hash of the [`Overlay`].
    ///
    /// The produced hash is used by the runtime to decide if the [`Layout`]
    /// needs to be recomputed between frames. Therefore, to ensure maximum
    /// efficiency, the hash should only be affected by the properties of the
    /// [`Overlay`] that can affect layouting.
    ///
    /// For example, the [`Text`] widget does not hash its color property, as
    /// its value cannot affect the overall [`Layout`] of the user interface.
    ///
    /// [`Overlay`]: trait.Overlay.html
    /// [`Layout`]: ../layout/struct.Layout.html
    /// [`Text`]: text/struct.Text.html
    fn hash_layout(&self, state: &mut Hasher, position: Point);

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
    ///
    /// [`Event`]: ../enum.Event.html
    /// [`Overlay`]: trait.Widget.html
    /// [`Layout`]: ../layout/struct.Layout.html
    /// [`Clipboard`]: ../trait.Clipboard.html
    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
    }
}
