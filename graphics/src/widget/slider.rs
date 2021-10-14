//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
use crate::{Backend, Renderer};
use iced_native::slider;

pub use iced_native::slider::State;
pub use iced_style::slider::{Handle, HandleShape, Style, StyleSheet};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// This is an alias of an `iced_native` slider with an `iced_wgpu::Renderer`.
pub type Slider<'a, T, Message, Backend> =
    iced_native::Slider<'a, T, Message, Renderer<Backend>>;

impl<B> slider::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_HEIGHT: u16 = 22;
}
