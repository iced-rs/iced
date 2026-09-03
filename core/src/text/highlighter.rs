//! Highlight text.
use crate::font;
use crate::{Color, Theme};

use std::ops::Range;

/// A type capable of highlighting text.
///
/// A [`Highlighter`] highlights lines in sequence. When a line changes,
/// it must be notified and the lines after the changed one must be fed
/// again to the [`Highlighter`].
pub trait Highlighter: 'static {
    /// The settings to configure the [`Highlighter`].
    type Settings: PartialEq + Clone;

    /// The highlight iterator type.
    type Iterator<'a>: Iterator<Item = (Range<usize>, Scope)>
    where
        Self: 'a;

    /// Creates a new [`Highlighter`] from its [`Self::Settings`].
    fn new(settings: &Self::Settings) -> Self;

    /// Updates the [`Highlighter`] with some new [`Self::Settings`].
    fn update(&mut self, new_settings: &Self::Settings);

    /// Notifies the [`Highlighter`] that the line at the given index has changed.
    fn change_line(&mut self, line: usize);

    /// Highlights the given line.
    ///
    /// If a line changed prior to this, the first line provided here will be the
    /// line that changed.
    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_>;

    /// Returns the current line of the [`Highlighter`].
    ///
    /// If `change_line` has been called, this will normally be the least index
    /// that changed.
    fn current_line(&self) -> usize;
}

/// A highlighter that highlights nothing.
#[derive(Debug, Clone, Copy)]
pub struct PlainText;

impl Highlighter for PlainText {
    type Settings = ();

    type Iterator<'a> = std::iter::Empty<(Range<usize>, Scope)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, _line: usize) {}

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        std::iter::empty()
    }

    fn current_line(&self) -> usize {
        usize::MAX
    }
}

/// The scope of a highlighted region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
    /// A comment.
    Comment,
    /// A string or character literal.
    String,
    /// A keyword or storage word.
    Keyword,
    /// A constant, numeric, or boolean literal.
    Constant,
    /// A function or method name.
    Function,
    /// A type, class, or tag name.
    Type,
    /// A variable.
    Variable,
    /// A built-in or support symbol.
    Support,
    /// Punctuation.
    Punctuation,
    /// A path component.
    Path,
    /// An invalid or erroneous construct.
    Invalid,
    /// Anything that does not match another class.
    Other,
}

/// A type that describes how to style the [`Scope`]s of a [`Highlighter`].
pub trait Highlight {
    /// Returns the [`Format`] of the given [`Scope`].
    fn highlight(&self, scope: Scope) -> Format;
}

impl Highlight for Theme {
    fn highlight(&self, scope: Scope) -> Format {
        let palette = self.palette();

        let color = match scope {
            Scope::Keyword => Some(palette.primary.base.color),
            Scope::Type | Scope::Path | Scope::Function => Some(palette.warning.base.color),

            Scope::Constant => Some(palette.danger.base.color),

            Scope::String => Some(palette.success.base.color),
            Scope::Support => Some(palette.primary.base.color),

            Scope::Punctuation => Some(palette.secondary.strong.color),
            Scope::Comment => Some(palette.secondary.base.color),

            Scope::Invalid => Some(palette.danger.base.color),
            Scope::Variable => Some(palette.danger.base.color),
            Scope::Other => None,
        };

        Format {
            color,
            style: (scope == Scope::Comment).then_some(font::Style::Italic),
        }
    }
}

/// The format of some text.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Format {
    /// The [`Color`] of the text.
    pub color: Option<Color>,
    /// The [`font::Style`] of the text.
    pub style: Option<font::Style>,
}
