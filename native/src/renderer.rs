//! Write your own renderer.
//!
//! You will need to implement the `Renderer` trait first. It simply contains
//! an `Output` associated type.
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
#[cfg(debug_assertions)]
mod null;
mod windowed;

pub use debugger::Debugger;
#[cfg(debug_assertions)]
pub use null::Null;
pub use windowed::{Target, Windowed};

use crate::{layout, Element};

pub trait Renderer: Sized {
    type Output;

    fn layout<'a, Message>(
        &mut self,
        element: &Element<'a, Message, Self>,
    ) -> layout::Node {
        element.layout(self, &layout::Limits::NONE)
    }
}
