use crate::{Color, Layout, Point, Renderer, Widget};

/// A renderer able to graphically explain a [`Layout`].
///
/// [`Layout`]: struct.Layout.html
pub trait Debugger: Renderer {
    /// Explains the [`Layout`] of an [`Element`] for debugging purposes.
    ///
    /// This will be called when [`Element::explain`] has been used. It should
    /// _explain_ the given [`Layout`] graphically.
    ///
    /// A common approach consists in recursively rendering the bounds of the
    /// [`Layout`] and its children.
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Element`]: ../struct.Element.html
    /// [`Element::explain`]: ../struct.Element.html#method.explain
    fn explain<Message>(
        &mut self,
        defaults: &Self::Defaults,
        widget: &dyn Widget<'_, Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        color: Color,
    ) -> Self::Output;
}
