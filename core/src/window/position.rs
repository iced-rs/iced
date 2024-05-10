use crate::{Point, Size};

/// The position of a window in a given screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    /// The platform-specific default position for a new window.
    Default,
    /// The window is completely centered on the screen.
    Centered,
    /// The window is positioned with specific coordinates: `(X, Y)`.
    ///
    /// When the decorations of the window are enabled, Windows 10 will add some
    /// invisible padding to the window. This padding gets included in the
    /// position. So if you have decorations enabled and want the window to be
    /// at (0, 0) you would have to set the position to
    /// `(PADDING_X, PADDING_Y)`.
    Specific(Point),
    /// Like [`Specific`], but the window is positioned with the specific coordinates returned by the function.
    ///
    /// The function receives the window size and the monitor's resolution as input.
    ///
    /// [`Specific`]: Self::Specific
    SpecificWith(fn(Size, Size) -> Point),
}

impl Default for Position {
    fn default() -> Self {
        Self::Default
    }
}
