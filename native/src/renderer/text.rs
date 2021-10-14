use crate::alignment;
use crate::{Color, Rectangle, Renderer};

pub trait Text: Renderer {
    /// The font type used.
    type Font: Default + Copy;

    fn fill_text(&mut self, section: Section<'_, Self::Font>);
}

#[derive(Debug, Clone, Copy)]
pub struct Section<'a, Font> {
    pub content: &'a str,
    pub bounds: Rectangle,
    pub size: Option<f32>,
    pub color: Option<Color>,
    pub font: Font,
    pub horizontal_alignment: alignment::Horizontal,
    pub vertical_alignment: alignment::Vertical,
}
