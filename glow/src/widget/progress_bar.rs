//! Display progress to your uses with a bar
use crate::Renderer;

pub use iced_graphics::progress_bar::{Style, StyleSheet};

/// A bar that displays progress.
///
/// This is an alias of an `iced_native` progress bar with an
/// `iced_wgpu::Renderer`.
pub type ProgressBar = iced_native::ProgressBar<Renderer>;
