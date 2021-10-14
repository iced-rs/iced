//! Show toggle controls using checkboxes.
use crate::backend::{self, Backend};
use crate::Renderer;

use iced_native::checkbox;

pub use iced_style::checkbox::{Style, StyleSheet};

/// A box that can be checked.
///
/// This is an alias of an `iced_native` checkbox with an `iced_wgpu::Renderer`.
pub type Checkbox<Message, Backend> =
    iced_native::Checkbox<Message, Renderer<Backend>>;

impl<B> checkbox::Renderer for Renderer<B>
where
    B: Backend + backend::Text,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_SIZE: u16 = 20;
    const DEFAULT_SPACING: u16 = 15;
}
