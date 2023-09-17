use crate::Color;

use std::hash::Hash;
use std::ops::Range;

pub trait Highlighter: 'static {
    type Settings: Hash;
    type Highlight;

    type Iterator<'a>: Iterator<Item = (Range<usize>, Self::Highlight)>
    where
        Self: 'a;

    fn new(settings: &Self::Settings) -> Self;

    fn change_line(&mut self, line: usize);

    fn highlight_line(&mut self, line: &str) -> Self::Iterator<'_>;

    fn current_line(&self) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub color: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct PlainText;

impl Highlighter for PlainText {
    type Settings = ();
    type Highlight = ();

    type Iterator<'a> = std::iter::Empty<(Range<usize>, Self::Highlight)>;

    fn new(_settings: &Self::Settings) -> Self {
        Self
    }

    fn change_line(&mut self, _line: usize) {}

    fn highlight_line(&mut self, _line: &str) -> Self::Iterator<'_> {
        std::iter::empty()
    }

    fn current_line(&self) -> usize {
        usize::MAX
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Format<Font> {
    pub color: Option<Color>,
    pub font: Option<Font>,
}
