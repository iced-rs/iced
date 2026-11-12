//! Highlight text.
use crate::Color;
use crate::font;

/// A type that describes how to highlight an `Input`
/// with some [`Style`].
pub trait Highlighter<Input, Theme = crate::Theme> {
    /// A unique identifier for the highlighter.
    fn id(&self) -> &str;

    /// Returns the [`Style`] of the given `Input`.
    fn highlight(&self, input: Input, theme: &Theme) -> Style;
}

/// The style of some highlighted text.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    /// The [`Color`] of the text.
    pub color: Option<Color>,
    /// The [`font::Style`] of the text.
    pub style: Option<font::Style>,
}

impl<T, Input, Theme> Highlighter<Input, Theme> for T
where
    T: Fn(Input, &Theme) -> Style,
{
    fn id(&self) -> &str {
        std::any::type_name_of_val(self) // Hack: Best effort
    }

    fn highlight(&self, input: Input, theme: &Theme) -> Style {
        (self)(input, theme)
    }
}
