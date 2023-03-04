//! Draw and interact with text.
use crate::alignment;
use crate::{Color, Point, Rectangle, Size};

use std::borrow::Cow;

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
}

impl Hit {
    /// Computes the cursor position of the [`Hit`] .
    pub fn cursor(self) -> usize {
        match self {
            Self::CharOffset(i) => i,
        }
    }
}

/// A renderer capable of measuring and drawing [`Text`].
pub trait Renderer: crate::Renderer {
    /// The font type used.
    type Font: Copy;

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

    /// Returns the default [`Self::Font`].
    fn default_font(&self) -> Self::Font;

    /// Returns the default size of [`Text`].
    fn default_size(&self) -> f32;

    /// Measures the text in the given bounds and returns the minimum boundaries
    /// that can fit the contents.
    fn measure(
        &self,
        content: &str,
        size: f32,
        font: Self::Font,
        bounds: Size,
    ) -> (f32, f32);

    /// Measures the width of the text as if it were laid out in a single line.
    fn measure_width(&self, content: &str, size: f32, font: Self::Font) -> f32 {
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

    /// Loads a [`Self::Font`] from its bytes.
    fn load_font(&mut self, font: Cow<'static, [u8]>);

    /// Draws the given [`Text`].
    fn fill_text(&mut self, text: Text<'_, Self::Font>);
}
