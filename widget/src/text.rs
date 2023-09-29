//! Draw and interact with text.
pub use crate::core::widget::text::*;

/// A paragraph.
///
/// # Example
///
/// ```
#[doc = include_str!("../../examples/hello_world/src/main.rs")]
/// ```
#[doc(alias("Label"))]
pub type Text<'a, Renderer = crate::Renderer> =
    crate::core::widget::Text<'a, Renderer>;
