//! Draw and interact with text.
pub use crate::core::widget::text::*;

/// A paragraph.
pub type Text<'a, Renderer = crate::Renderer> =
    crate::core::widget::Text<'a, Renderer>;
