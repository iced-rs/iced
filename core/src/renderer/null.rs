use crate::renderer::{self, Renderer};
use crate::text::{self, Text};
use crate::{Background, Font, Point, Rectangle, Size, Vector};

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

    const ICON_FONT: Font = Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';

    fn default_font(&self) -> Self::Font {
        Font::default()
    }

    fn default_size(&self) -> f32 {
        16.0
    }

    fn load_font(&mut self, _font: Cow<'static, [u8]>) {}

    fn measure(
        &self,
        _content: &str,
        _size: f32,
        _line_height: text::LineHeight,
        _font: Font,
        _bounds: Size,
        _shaping: text::Shaping,
    ) -> (f32, f32) {
        (0.0, 20.0)
    }

    fn hit_test(
        &self,
        _contents: &str,
        _size: f32,
        _line_height: text::LineHeight,
        _font: Self::Font,
        _bounds: Size,
        _shaping: text::Shaping,
        _point: Point,
        _nearest_only: bool,
    ) -> Option<text::Hit> {
        None
    }

    fn fill_text(&mut self, _text: Text<'_, Self::Font>) {}
}
