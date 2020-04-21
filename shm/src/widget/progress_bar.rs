//! Allow your users to perform actions by pressing a button.
//!
//! A [`Button`] has some local [`State`].
//!
//! [`Button`]: type.Button.html
//! [`State`]: struct.State.html
use crate::Renderer;

pub use iced_style::progress_bar::{Style, StyleSheet};

/// A bar that displays progress.
///
/// This is an alias of an `iced_native` progress bar with an
/// `iced_wgpu::Renderer`.
pub type ProgressBar = iced_native::ProgressBar<Renderer>;
