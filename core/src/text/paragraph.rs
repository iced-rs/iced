use crate::alignment;
use crate::text::{Difference, Hit, Text};
use crate::{Point, Size};

/// A text paragraph.
pub trait Paragraph: Sized + Default {
    /// The font of this [`Paragraph`].
    type Font: Copy + PartialEq;

    /// Creates a new [`Paragraph`] laid out with the given [`Text`].
    fn with_text(text: Text<&str, Self::Font>) -> Self;

    /// Lays out the [`Paragraph`] with some new boundaries.
    fn resize(&mut self, new_bounds: Size);

    /// Compares the [`Paragraph`] with some desired [`Text`] and returns the
    /// [`Difference`].
    fn compare(&self, text: Text<&str, Self::Font>) -> Difference;

    /// Returns the horizontal alignment of the [`Paragraph`].
    fn horizontal_alignment(&self) -> alignment::Horizontal;

    /// Returns the vertical alignment of the [`Paragraph`].
    fn vertical_alignment(&self) -> alignment::Vertical;

    /// Returns the minimum boundaries that can fit the contents of the
    /// [`Paragraph`].
    fn min_bounds(&self) -> Size;

    /// Tests whether the provided point is within the boundaries of the
    /// [`Paragraph`], returning information about the nearest character.
    fn hit_test(&self, point: Point) -> Option<Hit>;

    /// Returns the distance to the given grapheme index in the [`Paragraph`].
    fn grapheme_position(&self, line: usize, index: usize) -> Option<Point>;

    /// Updates the [`Paragraph`] to match the given [`Text`], if needed.
    fn update(&mut self, text: Text<&str, Self::Font>) {
        match self.compare(text) {
            Difference::None => {}
            Difference::Bounds => {
                self.resize(text.bounds);
            }
            Difference::Shape => {
                *self = Self::with_text(text);
            }
        }
    }

    /// Returns the minimum width that can fit the contents of the [`Paragraph`].
    fn min_width(&self) -> f32 {
        self.min_bounds().width
    }

    /// Returns the minimum height that can fit the contents of the [`Paragraph`].
    fn min_height(&self) -> f32 {
        self.min_bounds().height
    }
}
