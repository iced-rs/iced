use crate::renderer::{self, Renderer};
use crate::text::{self, Text};
use crate::{Background, Font, Point, Rectangle, Size, Vector};

/// A renderer that does nothing.
///
/// It can be useful if you are writing tests!
#[derive(Debug, Clone, Copy)]
pub struct Null;

impl Null {
    /// Creates a new [`Null`] renderer.
    pub fn new() -> Null {
        Null
    }
}

impl Renderer for Null {
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

    const ICON_FONT: Font = Font::Default;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';

    fn default_size(&self) -> u16 {
        20
    }

    fn measure(
        &self,
        _content: &str,
        _size: u16,
        _font: Font,
        _bounds: Size,
    ) -> (f32, f32) {
        (0.0, 20.0)
    }

    fn hit_test(
        &self,
        _contents: &str,
        _size: f32,
        _font: Self::Font,
        _bounds: Size,
        _point: Point,
        _nearest_only: bool,
    ) -> Option<text::Hit> {
        None
    }

    fn fill_text(&mut self, _text: Text<'_, Self::Font>) {}
}
