//! Create choices using radio buttons.
use crate::{Backend, Renderer};
use iced_native::radio;

pub use iced_style::radio::{Style, StyleSheet};

/// A circular button representing a choice.
///
/// This is an alias of an `iced_native` radio button with an
/// `iced_wgpu::Renderer`.
pub type Radio<Message, Backend> =
    iced_native::Radio<Message, Renderer<Backend>>;

impl<B> radio::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_SIZE: u16 = 28;
    const DEFAULT_SPACING: u16 = 15;
}
