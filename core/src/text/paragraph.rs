//! Draw paragraphs.
use crate::alignment;
use crate::text::{Alignment, Difference, Hit, Span, Text};
use crate::{Point, Rectangle, Size};

/// A text paragraph.
pub trait Paragraph: Sized + Default {
    /// The font of this [`Paragraph`].
    type Font: Copy + PartialEq;

    /// Creates a new [`Paragraph`] laid out with the given [`Text`].
    fn with_text(text: Text<&str, Self::Font>) -> Self;

    /// Creates a new [`Paragraph`] laid out with the given [`Text`].
    fn with_spans<Link>(
        text: Text<&[Span<'_, Link, Self::Font>], Self::Font>,
    ) -> Self;

    /// Lays out the [`Paragraph`] with some new boundaries.
    fn resize(&mut self, new_bounds: Size);

    /// Compares the [`Paragraph`] with some desired [`Text`] and returns the
    /// [`Difference`].
    fn compare(&self, text: Text<(), Self::Font>) -> Difference;

    /// Returns the horizontal alignment of the [`Paragraph`].
    fn align_x(&self) -> Alignment;

    /// Returns the vertical alignment of the [`Paragraph`].
    fn align_y(&self) -> alignment::Vertical;

    /// Returns the minimum boundaries that can fit the contents of the
    /// [`Paragraph`].
    fn min_bounds(&self) -> Size;

    /// Tests whether the provided point is within the boundaries of the
    /// [`Paragraph`], returning information about the nearest character.
    fn hit_test(&self, point: Point) -> Option<Hit>;

    /// Tests whether the provided point is within the boundaries of a
    /// [`Span`] in the [`Paragraph`], returning the index of the [`Span`]
    /// that was hit.
    fn hit_span(&self, point: Point) -> Option<usize>;

    /// Returns all bounds for the provided [`Span`] index of the [`Paragraph`].
    /// A [`Span`] can have multiple bounds for each line it's on.
    fn span_bounds(&self, index: usize) -> Vec<Rectangle>;

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

/// A [`Paragraph`] of plain text.
#[derive(Debug, Clone, Default)]
pub struct Plain<P: Paragraph> {
    raw: P,
    content: String,
}

impl<P: Paragraph> Plain<P> {
    /// Creates a new [`Plain`] paragraph.
    pub fn new(text: Text<&str, P::Font>) -> Self {
        let content = text.content.to_owned();

        Self {
            raw: P::with_text(text),
            content,
        }
    }

    /// Updates the plain [`Paragraph`] to match the given [`Text`], if needed.
    pub fn update(&mut self, text: Text<&str, P::Font>) {
        if self.content != text.content {
            text.content.clone_into(&mut self.content);
            self.raw = P::with_text(text);
            return;
        }

        match self.raw.compare(Text {
            content: (),
            bounds: text.bounds,
            size: text.size,
            line_height: text.line_height,
            font: text.font,
            align_x: text.align_x,
            align_y: text.align_y,
            shaping: text.shaping,
            wrapping: text.wrapping,
        }) {
            Difference::None => {}
            Difference::Bounds => {
                self.raw.resize(text.bounds);
            }
            Difference::Shape => {
                self.raw = P::with_text(text);
            }
        }
    }

    /// Returns the horizontal alignment of the [`Paragraph`].
    pub fn align_x(&self) -> Alignment {
        self.raw.align_x()
    }

    /// Returns the vertical alignment of the [`Paragraph`].
    pub fn align_y(&self) -> alignment::Vertical {
        self.raw.align_y()
    }

    /// Returns the minimum boundaries that can fit the contents of the
    /// [`Paragraph`].
    pub fn min_bounds(&self) -> Size {
        self.raw.min_bounds()
    }

    /// Returns the minimum width that can fit the contents of the
    /// [`Paragraph`].
    pub fn min_width(&self) -> f32 {
        self.raw.min_width()
    }

    /// Returns the minimum height that can fit the contents of the
    /// [`Paragraph`].
    pub fn min_height(&self) -> f32 {
        self.raw.min_height()
    }

    /// Returns the cached [`Paragraph`].
    pub fn raw(&self) -> &P {
        &self.raw
    }
}
