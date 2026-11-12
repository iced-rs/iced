//! Parse text.
use std::ops::Range;

/// A type capable of parsing text, producing some `Output`.
///
/// A [`Parser`] parses lines in sequence. When a line changes,
/// it must be notified and the lines after the changed one must be fed
/// again to the [`Parser`].
pub trait Parser: 'static {
    /// The settings to configure the [`Parser`].
    type Settings: PartialEq + Clone;

    /// The output of this [`Parser`].
    type Output;

    /// The parse iterator type.
    type Iterator<'a>: Iterator<Item = (Range<usize>, Self::Output)>
    where
        Self: 'a;

    /// Creates a new [`Parser`] from its [`Self::Settings`].
    fn new(settings: &Self::Settings) -> Self;

    /// Updates the [`Parser`] with some new [`Self::Settings`].
    fn update(&mut self, new_settings: &Self::Settings);

    /// Notifies the [`Parser`] that the line at the given index has changed.
    fn change_line(&mut self, line: usize);

    /// Parses the given line.
    ///
    /// If a line changed prior to this, the first line provided here will be the
    /// line that changed.
    fn parse_line(&mut self, line: &str) -> Self::Iterator<'_>;

    /// Returns the current line of the [`Parser`].
    ///
    /// If `change_line` has been called, this will normally be the least index
    /// that changed.
    fn current_line(&self) -> usize;
}

/// A parser that produces no output.
#[derive(Debug, Clone, Copy)]
pub struct PlainText;

impl Parser for PlainText {
    type Settings = ();
    type Output = ();

    type Iterator<'a> = std::iter::Empty<(Range<usize>, ())>;

    fn new(_settings: &Self::Settings) -> Self {
        Self
    }

    fn update(&mut self, _new_settings: &Self::Settings) {}

    fn change_line(&mut self, _line: usize) {}

    fn parse_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        std::iter::empty()
    }

    fn current_line(&self) -> usize {
        usize::MAX
    }
}
