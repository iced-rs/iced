//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
use crate::{Backend, Renderer};
use iced_native::Padding;

pub use iced_native::button::State;
pub use iced_style::button::{Style, StyleSheet};

/// A widget that produces a message when clicked.
///
/// This is an alias of an `iced_native` button with an `iced_wgpu::Renderer`.
pub type Button<'a, Message, Backend> =
    iced_native::Button<'a, Message, Renderer<Backend>>;

impl<B> iced_native::button::Renderer for Renderer<B>
where
    B: Backend,
{
    const DEFAULT_PADDING: Padding = Padding::new(5);

    type Style = Box<dyn StyleSheet>;
}
