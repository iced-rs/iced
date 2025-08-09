//! Edit text.
use crate::text::highlighter::{self, Highlighter};
use crate::text::{LineHeight, Wrapping};
use crate::{Pixels, Point, Rectangle, Size};

use std::borrow::Cow;
use std::sync::Arc;

/// A component that can be used by widgets to edit multi-line text.
pub trait Editor: Sized + Default {
    /// The font of the [`Editor`].
    type Font: Copy + PartialEq + Default;

    /// Creates a new [`Editor`] laid out with the given text.
    fn with_text(text: &str) -> Self;

    /// Returns true if the [`Editor`] has no contents.
    fn is_empty(&self) -> bool;

    /// Returns the current [`Cursor`] of the [`Editor`].
    fn cursor(&self) -> Cursor;

    /// Returns the current cursor position of the [`Editor`].
    ///
    /// Line and column, respectively.
    fn cursor_position(&self) -> (usize, usize);

    /// Returns the current selected text of the [`Editor`].
    fn selection(&self) -> Option<String>;

    /// Returns the text of the given line in the [`Editor`], if it exists.
    fn line(&self, index: usize) -> Option<Line<'_>>;

    /// Returns the amount of lines in the [`Editor`].
    fn line_count(&self) -> usize;

    /// Performs an [`Action`] on the [`Editor`].
    fn perform(&mut self, action: Action);

    /// Returns the current boundaries of the [`Editor`].
    fn bounds(&self) -> Size;

    /// Returns the minimum boundaries to fit the current contents of
    /// the [`Editor`].
    fn min_bounds(&self) -> Size;

    /// Updates the [`Editor`] with some new attributes.
    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Self::Font,
        new_size: Pixels,
        new_line_height: LineHeight,
        new_wrapping: Wrapping,
        new_highlighter: &mut impl Highlighter,
    );

    /// Runs a text [`Highlighter`] in the [`Editor`].
    fn highlight<H: Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> highlighter::Format<Self::Font>,
    );
}

/// An interaction with an [`Editor`].
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Apply a [`Motion`].
    Move(Motion),
    /// Select text with a given [`Motion`].
    Select(Motion),
    /// Select the word at the current cursor.
    SelectWord,
    /// Select the line at the current cursor.
    SelectLine,
    /// Select the entire buffer.
    SelectAll,
    /// Perform an [`Edit`].
    Edit(Edit),
    /// Click the [`Editor`] at the given [`Point`].
    Click(Point),
    /// Drag the mouse on the [`Editor`] to the given [`Point`].
    Drag(Point),
    /// Scroll the [`Editor`] a certain amount of lines.
    Scroll {
        /// The amount of lines to scroll.
        lines: i32,
    },
}

impl Action {
    /// Returns whether the [`Action`] is an editing action.
    pub fn is_edit(&self) -> bool {
        matches!(self, Self::Edit(_))
    }
}

/// An action that edits text.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Insert the given character.
    Insert(char),
    /// Paste the given text.
    Paste(Arc<String>),
    /// Break the current line.
    Enter,
    /// Indent the current line.
    Indent,
    /// Unindent the current line.
    Unindent,
    /// Delete the previous character.
    Backspace,
    /// Delete the next character.
    Delete,
}

/// A cursor movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    /// Move left.
    Left,
    /// Move right.
    Right,
    /// Move up.
    Up,
    /// Move down.
    Down,
    /// Move to the left boundary of a word.
    WordLeft,
    /// Move to the right boundary of a word.
    WordRight,
    /// Move to the start of the line.
    Home,
    /// Move to the end of the line.
    End,
    /// Move to the start of the previous window.
    PageUp,
    /// Move to the start of the next window.
    PageDown,
    /// Move to the start of the text.
    DocumentStart,
    /// Move to the end of the text.
    DocumentEnd,
}

impl Motion {
    /// Widens the [`Motion`], if possible.
    pub fn widen(self) -> Self {
        match self {
            Self::Left => Self::WordLeft,
            Self::Right => Self::WordRight,
            Self::Home => Self::DocumentStart,
            Self::End => Self::DocumentEnd,
            _ => self,
        }
    }

    /// Returns the [`Direction`] of the [`Motion`].
    pub fn direction(&self) -> Direction {
        match self {
            Self::Left
            | Self::Up
            | Self::WordLeft
            | Self::Home
            | Self::PageUp
            | Self::DocumentStart => Direction::Left,
            Self::Right
            | Self::Down
            | Self::WordRight
            | Self::End
            | Self::PageDown
            | Self::DocumentEnd => Direction::Right,
        }
    }
}

/// A direction in some text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// <-
    Left,
    /// ->
    Right,
}

/// The cursor of an [`Editor`].
#[derive(Debug, Clone)]
pub enum Cursor {
    /// Cursor without a selection
    Caret(Point),

    /// Cursor selecting a range of text
    Selection(Vec<Rectangle>),
}

/// A line of an [`Editor`].
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Line<'a> {
    /// The raw text of the [`Line`].
    pub text: Cow<'a, str>,
    /// The line ending of the [`Line`].
    pub ending: LineEnding,
}

/// The line ending of a [`Line`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LineEnding {
    /// Use `\n` for line ending (POSIX-style)
    #[default]
    Lf,
    /// Use `\r\n` for line ending (Windows-style)
    CrLf,
    /// Use `\r` for line ending (many legacy systems)
    Cr,
    /// Use `\n\r` for line ending (some legacy systems)
    LfCr,
    /// No line ending
    None,
}

impl LineEnding {
    /// Gets the string representation of the [`LineEnding`].
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
            Self::LfCr => "\n\r",
            Self::None => "",
        }
    }
}
