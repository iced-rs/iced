//! Write your own renderer!
//!
//! There is not a common entrypoint or trait for a __renderer__ in Iced.
//! Instead, every [`Widget`] constrains its generic `Renderer` type as
//! necessary.
//!
//! This approach is flexible and composable. For instance, the
//! [`Text`] widget only needs a [`text::Renderer`] while a [`Checkbox`] widget
//! needs both a [`text::Renderer`] and a [`checkbox::Renderer`], reusing logic.
//!
//! In the end, a __renderer__ satisfying all the constraints is
//! needed to build a [`UserInterface`].
//!
//! [`Widget`]: ../widget/trait.Widget.html
//! [`UserInterface`]: ../struct.UserInterface.html
//! [`Text`]: ../widget/text/struct.Text.html
//! [`text::Renderer`]: ../widget/text/trait.Renderer.html
//! [`Checkbox`]: ../widget/checkbox/struct.Checkbox.html
//! [`checkbox::Renderer`]: ../widget/checkbox/trait.Renderer.html
use crate::Layout;

/// A renderer able to graphically explain a [`Layout`].
///
/// [`Layout`]: ../struct.Layout.html
pub trait Debugger {
    /// The color type that will be used to configure the _explanation_.
    ///
    /// This is the type that will be asked in [`Element::explain`].
    ///
    /// [`Element::explain`]: ../struct.Element.html#method.explain
    type Color: Copy;

    /// Explains the [`Layout`] of an [`Element`] for debugging purposes.
    ///
    /// This will be called when [`Element::explain`] has been used. It should
    /// _explain_ the given [`Layout`] graphically.
    ///
    /// A common approach consists in recursively rendering the bounds of the
    /// [`Layout`] and its children.
    ///
    /// [`Layout`]: struct.Layout.html
    /// [`Element`]: struct.Element.html
    /// [`Element::explain`]: struct.Element.html#method.explain
    fn explain(&mut self, layout: &Layout<'_>, color: Self::Color);
}
