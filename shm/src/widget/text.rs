//! Write some text for your users to read.
use crate::Renderer;

/// A paragraph of text.
///
/// This is an alias of an `iced_native` text with an `iced_wgpu::Renderer`.
pub type Text = iced_native::Text<Renderer>;
