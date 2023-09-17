use crate::text::highlighter::{self, Highlighter};
use crate::text::LineHeight;
use crate::{Pixels, Point, Rectangle, Size};

use std::sync::Arc;

pub trait Editor: Sized + Default {
    type Font: Copy + PartialEq + Default;

    /// Creates a new [`Editor`] laid out with the given text.
    fn with_text(text: &str) -> Self;

    fn cursor(&self) -> Cursor;

    fn selection(&self) -> Option<String>;

    fn line(&self, index: usize) -> Option<&str>;

    fn line_count(&self) -> usize;

    fn perform(&mut self, action: Action);

    /// Returns the current boundaries of the [`Editor`].
    fn bounds(&self) -> Size;

    /// Updates the [`Editor`] with some new attributes.
    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Self::Font,
        new_size: Pixels,
        new_line_height: LineHeight,
        new_highlighter: &mut impl Highlighter,
    );

    fn highlight<H: Highlighter>(
        &mut self,
        font: Self::Font,
        highlighter: &mut H,
        format_highlight: impl Fn(&H::Highlight) -> highlighter::Format<Self::Font>,
    );
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Move(Motion),
    Select(Motion),
    SelectWord,
    SelectLine,
    Edit(Edit),
    Click(Point),
    Drag(Point),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    Insert(char),
    Paste(Arc<String>),
    Enter,
    Backspace,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    WordLeft,
    WordRight,
    Home,
    End,
    PageUp,
    PageDown,
    DocumentStart,
    DocumentEnd,
}

impl Motion {
    pub fn widen(self) -> Self {
        match self {
            Self::Left => Self::WordLeft,
            Self::Right => Self::WordRight,
            Self::Home => Self::DocumentStart,
            Self::End => Self::DocumentEnd,
            _ => self,
        }
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
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
