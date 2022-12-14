/// The position of a window in a given screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Specific(i32, i32),
}

impl Default for Position {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(feature = "winit")]
impl From<Position> for iced_winit::Position {
    fn from(position: Position) -> Self {
        match position {
            Position::Default => Self::Default,
            Position::Centered => Self::Centered,
            Position::Specific(x, y) => Self::Specific(x, y),
        }
    }
}
