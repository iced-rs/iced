//! Draw and interact with text.
pub use crate::core::widget::text::*;

/// A paragraph.
pub type Text<'a, Theme = crate::Theme, Renderer = crate::Renderer> =
    crate::core::widget::Text<'a, Theme, Renderer>;
