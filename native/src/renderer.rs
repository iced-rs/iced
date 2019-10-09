//! Write your own renderer.
//!
//! You will need to implement the `Renderer` trait first. It simply contains
//! a `Primitive` associated type.
//!
//! There is no common trait to draw all the widgets. Instead, every [`Widget`]
//! constrains its generic `Renderer` type as necessary.
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

mod debugger;
mod windowed;

pub use debugger::Debugger;
pub use windowed::Windowed;

pub trait Renderer {
    type Primitive;
}
