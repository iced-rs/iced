use crate::Layout;

pub trait Renderer {
    type Color: Copy;

    /// Explains the [`Layout`] of an [`Element`] for debugging purposes.
    ///
    /// This will be called when [`Element::explain`] has been used. It should
    /// _explain_ the [`Layout`] graphically.
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Element`]: struct.Element.html
    /// [`Element::explain`]: struct.Element.html#method.explain
    fn explain(&mut self, layout: &Layout<'_>, color: Self::Color);
}
