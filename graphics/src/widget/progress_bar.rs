//! Allow your users to visually track the progress of a computation.
//!
//! A [`ProgressBar`] has a range of possible values and a current value,
//! as well as a length, height and style.
use crate::{Backend, Renderer};
use iced_native::progress_bar;

pub use iced_style::progress_bar::{Style, StyleSheet};

/// A bar that displays progress.
///
/// This is an alias of an `iced_native` progress bar with an
/// `iced_wgpu::Renderer`.
pub type ProgressBar<Backend> = iced_native::ProgressBar<Renderer<Backend>>;

impl<B> progress_bar::Renderer for Renderer<B>
where
    B: Backend,
{
    type Style = Box<dyn StyleSheet>;

    const DEFAULT_HEIGHT: u16 = 30;
}
