//! Draw and interact with text.
mod rich;

pub use crate::core::text::{Fragment, IntoFragment, Span};
pub use crate::core::widget::text::*;
pub use rich::Rich;

/// A paragraph.
pub type Text<'a, Theme = crate::Theme, Renderer = crate::Renderer> =
    crate::core::widget::Text<'a, Theme, Renderer>;
