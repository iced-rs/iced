//! Draw and interact with text.
use crate::alignment;
use crate::{Color, Point, Rectangle, Size, Vector};

/// A paragraph.
#[derive(Debug, Clone, Copy)]
pub struct Text<'a, Font> {
    /// The content of the paragraph.
    pub content: &'a str,

    /// The bounds of the paragraph.
    pub bounds: Rectangle,

    /// The size of the [`Text`].
    pub size: f32,

    /// The color of the [`Text`].
    pub color: Color,

    /// The font of the [`Text`].
    pub font: Font,

    /// The horizontal alignment of the [`Text`].
    pub horizontal_alignment: alignment::Horizontal,

    /// The vertical alignment of the [`Text`].
    pub vertical_alignment: alignment::Vertical,
}

/// The result of hit testing on text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hit {
    /// The point was within the bounds of the returned character index.
    CharOffset(usize),
    /// The provided point was not within the bounds of a glyph. The index
    /// of the character with the closest centeroid position is returned,
    /// as well as its delta.
    NearestCharOffset(usize, Vector),
}

impl Hit {
    /// Computes the cursor position corresponding to this [`HitTestResult`] .
    pub fn cursor(self) -> usize {
        match self {
            Self::CharOffset(i) => i,
            Self::NearestCharOffset(i, delta) => {
                if delta.x > f32::EPSILON {
                    i + 1
                } else {
                    i
                }
            }
        }
    }
}

/// A renderer capable of measuring and drawing [`Text`].
pub trait Renderer: crate::Renderer {
    /// The font type used.
    type Font: Default + Copy;

    /// The icon font of the backend.
    const ICON_FONT: Self::Font;

    /// The `char` representing a ✔ icon in the [`ICON_FONT`].
    ///
    /// [`ICON_FONT`]: Self::ICON_FONT
    const CHECKMARK_ICON: char;

    /// The `char` representing a ▼ icon in the built-in [`ICON_FONT`].
    ///
    /// [`ICON_FONT`]: Self::ICON_FONT
    const ARROW_DOWN_ICON: char;

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

    /// Measures the width of the text as if it were laid out in a single line.
    fn measure_width(&self, content: &str, size: u16, font: Self::Font) -> f32 {
        let (width, _) = self.measure(content, size, font, Size::INFINITY);

        width
    }

    /// Tests whether the provided point is within the boundaries of text
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

    /// Draws the given [`Text`].
    fn fill_text(&mut self, text: Text<'_, Self::Font>);
}
