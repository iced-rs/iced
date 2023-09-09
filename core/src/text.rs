//! Draw and interact with text.
use crate::alignment;
use crate::{Color, Pixels, Point, Size};

use std::borrow::Cow;
use std::hash::{Hash, Hasher};

/// A paragraph.
#[derive(Debug, Clone, Copy)]
pub struct Text<'a, Font> {
    /// The content of the paragraph.
    pub content: &'a str,

    /// The bounds of the paragraph.
    pub bounds: Size,

    /// The size of the [`Text`] in logical pixels.
    pub size: Pixels,

    /// The line height of the [`Text`].
    pub line_height: LineHeight,

    /// The font of the [`Text`].
    pub font: Font,

    /// The horizontal alignment of the [`Text`].
    pub horizontal_alignment: alignment::Horizontal,

    /// The vertical alignment of the [`Text`].
    pub vertical_alignment: alignment::Vertical,

    /// The [`Shaping`] strategy of the [`Text`].
    pub shaping: Shaping,
}

/// The shaping strategy of some text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Shaping {
    /// No shaping and no font fallback.
    ///
    /// This shaping strategy is very cheap, but it will not display complex
    /// scripts properly nor try to find missing glyphs in your system fonts.
    ///
    /// You should use this strategy when you have complete control of the text
    /// and the font you are displaying in your application.
    ///
    /// This is the default.
    #[default]
    Basic,
    /// Advanced text shaping and font fallback.
    ///
    /// You will need to enable this flag if the text contains a complex
    /// script, the font used needs it, and/or multiple fonts in your system
    /// may be needed to display all of the glyphs.
    ///
    /// Advanced shaping is expensive! You should only enable it when necessary.
    Advanced,
}

/// The height of a line of text in a paragraph.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineHeight {
    /// A factor of the size of the text.
    Relative(f32),

    /// An absolute height in logical pixels.
    Absolute(Pixels),
}

impl LineHeight {
    /// Returns the [`LineHeight`] in absolute logical pixels.
    pub fn to_absolute(self, text_size: Pixels) -> Pixels {
        match self {
            Self::Relative(factor) => Pixels(factor * text_size.0),
            Self::Absolute(pixels) => pixels,
        }
    }
}

impl Default for LineHeight {
    fn default() -> Self {
        Self::Relative(1.3)
    }
}

impl From<f32> for LineHeight {
    fn from(factor: f32) -> Self {
        Self::Relative(factor)
    }
}

impl From<Pixels> for LineHeight {
    fn from(pixels: Pixels) -> Self {
        Self::Absolute(pixels)
    }
}

impl Hash for LineHeight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Relative(factor) => {
                state.write_u8(0);
                factor.to_bits().hash(state);
            }
            Self::Absolute(pixels) => {
                state.write_u8(1);
                f32::from(*pixels).to_bits().hash(state);
            }
        }
    }
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
    type Font: Copy + PartialEq;

    /// The [`Paragraph`] of this [`Renderer`].
    type Paragraph: Paragraph<Font = Self::Font> + 'static;

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
    fn default_size(&self) -> Pixels;

    /// Loads a [`Self::Font`] from its bytes.
    fn load_font(&mut self, font: Cow<'static, [u8]>);

    /// Creates a new [`Paragraph`] laid out with the given [`Text`].
    fn create_paragraph(&self, text: Text<'_, Self::Font>) -> Self::Paragraph;

    /// Lays out the given [`Paragraph`] with some new boundaries.
    fn resize_paragraph(
        &self,
        paragraph: &mut Self::Paragraph,
        new_bounds: Size,
    );

    /// Updates a [`Paragraph`] to match the given [`Text`], if needed.
    fn update_paragraph(
        &self,
        paragraph: &mut Self::Paragraph,
        text: Text<'_, Self::Font>,
    ) {
        match compare(paragraph, text) {
            Difference::None => {}
            Difference::Bounds => {
                self.resize_paragraph(paragraph, text.bounds);
            }
            Difference::Shape => {
                *paragraph = self.create_paragraph(text);
            }
        }
    }

    /// Draws the given [`Paragraph`] at the given position and with the given
    /// [`Color`].
    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
    );

    /// Draws the given [`Text`] at the given position and with the given
    /// [`Color`].
    fn fill_text(
        &mut self,
        text: Text<'_, Self::Font>,
        position: Point,
        color: Color,
    );
}
/// A text paragraph.
pub trait Paragraph: Default {
    /// The font of this [`Paragraph`].
    type Font;

    /// Returns the content of the [`Paragraph`].
    fn content(&self) -> &str;

    /// Returns the text size of the [`Paragraph`].
    fn text_size(&self) -> Pixels;

    /// Returns the [`LineHeight`] of the [`Paragraph`].
    fn line_height(&self) -> LineHeight;

    /// Returns the [`Self::Font`] of the [`Paragraph`].
    fn font(&self) -> Self::Font;

    /// Returns the [`Shaping`] strategy of the [`Paragraph`].
    fn shaping(&self) -> Shaping;

    /// Returns the horizontal alignment of the [`Paragraph`].
    fn horizontal_alignment(&self) -> alignment::Horizontal;

    /// Returns the vertical alignment of the [`Paragraph`].
    fn vertical_alignment(&self) -> alignment::Vertical;

    /// Returns the boundaries of the [`Paragraph`].
    fn bounds(&self) -> Size;

    /// Returns the minimum boundaries that can fit the contents of the
    /// [`Paragraph`].
    fn min_bounds(&self) -> Size;

    /// Tests whether the provided point is within the boundaries of the
    /// [`Paragraph`], returning information about the nearest character.
    fn hit_test(&self, point: Point) -> Option<Hit>;

    /// Returns the distance to the given grapheme index in the [`Paragraph`].
    fn grapheme_position(&self, line: usize, index: usize) -> Option<Point>;

    /// Returns the minimum width that can fit the contents of the [`Paragraph`].
    fn min_width(&self) -> f32 {
        self.min_bounds().width
    }

    /// Returns the minimum height that can fit the contents of the [`Paragraph`].
    fn min_height(&self) -> f32 {
        self.min_bounds().height
    }
}

/// The difference detected in some text.
///
/// You will obtain a [`Difference`] when you [`compare`] a [`Paragraph`] with some
/// [`Text`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difference {
    /// No difference.
    ///
    /// The text can be reused as it is!
    None,

    /// A bounds difference.
    ///
    /// This normally means a relayout is necessary, but the shape of the text can
    /// be reused.
    Bounds,

    /// A shape difference.
    ///
    /// The contents, alignment, sizes, fonts, or any other essential attributes
    /// of the shape of the text have changed. A complete reshape and relayout of
    /// the text is necessary.
    Shape,
}

/// Compares a [`Paragraph`] with some desired [`Text`] and returns the
/// [`Difference`].
pub fn compare<Font: PartialEq>(
    paragraph: &impl Paragraph<Font = Font>,
    text: Text<'_, Font>,
) -> Difference {
    if paragraph.content() != text.content
        || paragraph.text_size() != text.size
        || paragraph.line_height().to_absolute(text.size)
            != text.line_height.to_absolute(text.size)
        || paragraph.font() != text.font
        || paragraph.shaping() != text.shaping
        || paragraph.horizontal_alignment() != text.horizontal_alignment
        || paragraph.vertical_alignment() != text.vertical_alignment
    {
        Difference::Shape
    } else if paragraph.bounds() != text.bounds {
        Difference::Bounds
    } else {
        Difference::None
    }
}
