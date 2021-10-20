use crate::alignment;
use crate::{Color, Point, Rectangle, Renderer, Size};

pub use crate::text::Hit;

pub trait Text: Renderer {
    /// The font type used.
    type Font: Default + Copy;

    /// Returns the default size of [`Text`].
    fn default_size(&self) -> u16;

    /// Measures the text in the given bounds and returns the minimum boundaries
    /// that can fit the contents.
    fn measure(
        &self,
        content: &str,
        size: u16,
        font: Self::Font,
        bounds: Size,
    ) -> (f32, f32);

    fn measure_width(&self, content: &str, size: u16, font: Self::Font) -> f32 {
        let (width, _) = self.measure(content, size, font, Size::INFINITY);

        width
    }

    /// Tests whether the provided point is within the boundaries of [`Text`]
    /// laid out with the given parameters, returning information about
    /// the nearest character.
    ///
    /// If `nearest_only` is true, the hit test does not consider whether the
    /// the point is interior to any glyph bounds, returning only the character
    /// with the nearest centeroid.
    fn hit_test(
        &self,
        contents: &str,
        size: f32,
        font: Self::Font,
        bounds: Size,
        point: Point,
        nearest_only: bool,
    ) -> Option<Hit>;

    fn fill_text(&mut self, section: Section<'_, Self::Font>);
}

#[derive(Debug, Clone, Copy)]
pub struct Section<'a, Font> {
    pub content: &'a str,
    pub bounds: Rectangle,
    pub size: f32,
    pub color: Color,
    pub font: Font,
    pub horizontal_alignment: alignment::Horizontal,
    pub vertical_alignment: alignment::Vertical,
}
