use crate::{Color, Layout, Point, Rectangle, Renderer, Widget};

/// A renderer able to graphically explain a [`Layout`].
pub trait Debugger: Renderer {
    /// Explains the [`Layout`] of an [`Element`] for debugging purposes.
    ///
    /// This will be called when [`Element::explain`] has been used. It should
    /// _explain_ the given [`Layout`] graphically.
    ///
    /// A common approach consists in recursively rendering the bounds of the
    /// [`Layout`] and its children.
    ///
    /// [`Element`]: crate::Element
    /// [`Element::explain`]: crate::Element::explain
    fn explain<Message>(
        &mut self,
        defaults: &Self::Defaults,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        color: Color,
    ) -> Self::Output;
}
