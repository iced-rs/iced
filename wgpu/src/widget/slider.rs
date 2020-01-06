//! Display an interactive selector of a single value from a range of values.
//!
//! A [`Slider`] has some local [`State`].
//!
//! [`Slider`]: struct.Slider.html
//! [`State`]: struct.State.html
use crate::Renderer;

pub use iced_native::slider::State;
pub use iced_style::slider::{Handle, HandleShape, Style, StyleSheet};

/// An horizontal bar and a handle that selects a single value from a range of
/// values.
///
/// This is an alias of an `iced_native` slider with an `iced_wgpu::Renderer`.
pub type Slider<'a, Message> = iced_native::Slider<'a, Message, Renderer>;
