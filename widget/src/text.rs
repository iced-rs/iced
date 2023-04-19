pub use crate::core::text::Shaping;
pub use crate::core::widget::text::*;

pub type Text<'a, Renderer = crate::Renderer> =
    crate::core::widget::Text<'a, Renderer>;
