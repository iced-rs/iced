use crate::text::LineHeight;
use crate::{Pixels, Point, Rectangle, Size};

pub trait Editor: Sized + Default {
    type Font: Copy + PartialEq + Default;

    /// Creates a new [`Editor`] laid out with the given text.
    fn with_text(text: &str) -> Self;

    fn cursor(&self) -> Cursor;

    fn perform(&mut self, action: Action);

    /// Returns the current boundaries of the [`Editor`].
    fn bounds(&self) -> Size;

    /// Returns the minimum boundaries that can fit the contents of the
    /// [`Editor`].
    fn min_bounds(&self) -> Size;

    /// Updates the [`Editor`] with some new attributes.
    fn update(
        &mut self,
        new_bounds: Size,
        new_font: Self::Font,
        new_size: Pixels,
        new_line_height: LineHeight,
    );

    /// Returns the minimum width that can fit the contents of the [`Editor`].
    fn min_width(&self) -> f32 {
        self.min_bounds().width
    }

    /// Returns the minimum height that can fit the contents of the [`Editor`].
    fn min_height(&self) -> f32 {
        self.min_bounds().height
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveLeftWord,
    MoveRightWord,
    MoveHome,
    MoveEnd,
    SelectWord,
    SelectLine,
    Insert(char),
    Enter,
    Backspace,
    Delete,
    Click(Point),
    Drag(Point),
}

/// The cursor of an [`Editor`].
#[derive(Debug, Clone)]
pub enum Cursor {
    /// Cursor without a selection
    Caret(Point),

    /// Cursor selecting a range of text
    Selection(Vec<Rectangle>),
}
