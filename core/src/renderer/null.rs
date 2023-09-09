use crate::alignment;
use crate::renderer::{self, Renderer};
use crate::text::{self, Text};
use crate::{Background, Color, Font, Pixels, Point, Rectangle, Size, Vector};

use std::borrow::Cow;

/// A renderer that does nothing.
///
/// It can be useful if you are writing tests!
#[derive(Debug, Clone, Copy, Default)]
pub struct Null;

impl Null {
    /// Creates a new [`Null`] renderer.
    pub fn new() -> Null {
        Null
    }
}

impl Renderer for Null {
    type Theme = ();

    fn with_layer(&mut self, _bounds: Rectangle, _f: impl FnOnce(&mut Self)) {}

    fn with_translation(
        &mut self,
        _translation: Vector,
        _f: impl FnOnce(&mut Self),
    ) {
    }

    fn clear(&mut self) {}

    fn fill_quad(
        &mut self,
        _quad: renderer::Quad,
        _background: impl Into<Background>,
    ) {
    }
}

impl text::Renderer for Null {
    type Font = Font;
    type Paragraph = ();

    const ICON_FONT: Font = Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';

    fn default_font(&self) -> Self::Font {
        Font::default()
    }

    fn default_size(&self) -> Pixels {
        Pixels(16.0)
    }

    fn load_font(&mut self, _font: Cow<'static, [u8]>) {}

    fn create_paragraph(&self, _text: Text<'_, Self::Font>) -> Self::Paragraph {
    }

    fn resize_paragraph(
        &self,
        _paragraph: &mut Self::Paragraph,
        _new_bounds: Size,
    ) {
    }

    fn fill_paragraph(
        &mut self,
        _paragraph: &Self::Paragraph,
        _position: Point,
        _color: Color,
    ) {
    }

    fn fill_text(
        &mut self,
        _paragraph: Text<'_, Self::Font>,
        _position: Point,
        _color: Color,
    ) {
    }
}

impl text::Paragraph for () {
    type Font = Font;

    fn content(&self) -> &str {
        ""
    }

    fn text_size(&self) -> Pixels {
        Pixels(16.0)
    }

    fn font(&self) -> Self::Font {
        Font::default()
    }

    fn line_height(&self) -> text::LineHeight {
        text::LineHeight::default()
    }

    fn shaping(&self) -> text::Shaping {
        text::Shaping::default()
    }

    fn horizontal_alignment(&self) -> alignment::Horizontal {
        alignment::Horizontal::Left
    }

    fn vertical_alignment(&self) -> alignment::Vertical {
        alignment::Vertical::Top
    }

    fn grapheme_position(&self, _line: usize, _index: usize) -> Option<Point> {
        None
    }

    fn bounds(&self) -> Size {
        Size::ZERO
    }

    fn min_bounds(&self) -> Size {
        Size::ZERO
    }

    fn hit_test(&self, _point: Point) -> Option<text::Hit> {
        None
    }
}
