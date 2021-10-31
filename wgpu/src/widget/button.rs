//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
use crate::Renderer;

pub use iced_graphics::button::{Style, StyleSheet};
pub use iced_native::widget::button::State;

/// A widget that produces a message when clicked.
///
/// This is an alias of an `iced_native` button with an `iced_wgpu::Renderer`.
pub type Button<'a, Message> =
    iced_native::widget::Button<'a, Message, Renderer>;
