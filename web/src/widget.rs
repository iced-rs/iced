//! Use the built-in widgets or create your own.
//!
//! # Custom widgets
//! If you want to implement a custom widget, you simply need to implement the
//! [`Widget`] trait. You can use the API of the built-in widgets as a guide or
//! source of inspiration.
//!
//! # Re-exports
//! For convenience, the contents of this module are available at the root
//! module. Therefore, you can directly type:
//!
//! ```
//! use iced_web::{button, Button, Widget};
//! ```
//!
//! [`Widget`]: trait.Widget.html
use crate::Bus;
use dodrio::bumpalo;

pub mod button;
pub mod slider;

mod checkbox;
mod column;
mod image;
mod radio;
mod row;
mod text;

#[doc(no_inline)]
pub use button::Button;

#[doc(no_inline)]
pub use slider::Slider;

#[doc(no_inline)]
pub use text::Text;

pub use checkbox::Checkbox;
pub use column::Column;
pub use image::Image;
pub use radio::Radio;
pub use row::Row;

/// A component that displays information and allows interaction.
///
/// If you want to build your own widgets, you will need to implement this
/// trait.
///
/// [`Widget`]: trait.Widget.html
pub trait Widget<Message> {
    /// Produces a VDOM node for the [`Widget`].
    ///
    /// [`Widget`]: trait.Widget.html
    fn node<'b>(
        &self,
        bump: &'b bumpalo::Bump,
        _bus: &Bus<Message>,
    ) -> dodrio::Node<'b>;
}
