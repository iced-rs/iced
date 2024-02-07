use crate::alignment;
use crate::renderer::{self, Renderer};
use crate::text::{self, Text};
use crate::{
    Background, Color, Font, Pixels, Point, Rectangle, Size, Transformation,
};

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
    fn with_layer(&mut self, _bounds: Rectangle, _f: impl FnOnce(&mut Self)) {}

    fn with_transformation(
        &mut self,
        _transformation: Transformation,
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
    type Editor = ();

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

    fn fill_paragraph(
        &mut self,
        _paragraph: &Self::Paragraph,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_editor(
        &mut self,
        _editor: &Self::Editor,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }

    fn fill_text(
        &mut self,
        _paragraph: Text<'_, Self::Font>,
        _position: Point,
        _color: Color,
        _clip_bounds: Rectangle,
    ) {
    }
}

impl text::Paragraph for () {
    type Font = Font;

    fn with_text(_text: Text<'_, Self::Font>) -> Self {}

    fn resize(&mut self, _new_bounds: Size) {}

    fn compare(&self, _text: Text<'_, Self::Font>) -> text::Difference {
        text::Difference::None
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

    fn min_bounds(&self) -> Size {
        Size::ZERO
    }

    fn hit_test(&self, _point: Point) -> Option<text::Hit> {
        None
    }
}

impl text::Editor for () {
    type Font = Font;

    fn with_text(_text: &str) -> Self {}

    fn cursor(&self) -> text::editor::Cursor {
        text::editor::Cursor::Caret(Point::ORIGIN)
    }

    fn cursor_position(&self) -> (usize, usize) {
        (0, 0)
    }

    fn selection(&self) -> Option<String> {
        None
    }

    fn line(&self, _index: usize) -> Option<&str> {
        None
    }

    fn line_count(&self) -> usize {
        0
    }

    fn perform(&mut self, _action: text::editor::Action) {}

    fn bounds(&self) -> Size {
        Size::ZERO
    }

    fn min_bounds(&self) -> Size {
        Size::ZERO
    }

    fn update(
        &mut self,
        _new_bounds: Size,
        _new_font: Self::Font,
        _new_size: Pixels,
        _new_line_height: text::LineHeight,
        _new_highlighter: &mut impl text::Highlighter,
    ) {
    }

    fn highlight<H: text::Highlighter>(
        &mut self,
        _font: Self::Font,
        _highlighter: &mut H,
        _format_highlight: impl Fn(
            &H::Highlight,
        ) -> text::highlighter::Format<Self::Font>,
    ) {
    }
}
