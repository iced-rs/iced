//! Highlight text.
use crate::font;
use crate::{Color, Theme};

/// A type that describes how to highlight an `Input`
/// with some [`Style`].
pub trait Highlighter<Input> {
    /// Returns the [`Style`] of the given `Input`.
    fn highlight(&self, input: Input) -> Style;
}

/// The style of some highlighted text.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    /// The [`Color`] of the text.
    pub color: Option<Color>,
    /// The [`font::Style`] of the text.
    pub style: Option<font::Style>,
}

impl Highlighter<()> for Theme {
    fn highlight(&self, _input: ()) -> Style {
        Style::default()
    }
}
