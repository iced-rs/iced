/// The cardinal directions relative to the center of a window.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Points to the top edge of a window.
    North,

    /// Points to the bottom edge of a window.
    South,

    /// Points to the right edge of a window.
    East,

    /// Points to the left edge of a window.
    West,

    /// Points to the top-right corner of a window.
    NorthEast,

    /// Points to the top-left corner of a window.
    NorthWest,

    /// Points to the bottom-right corner of a window.
    SouthEast,

    /// Points to the bottom-left corner of a window.
    SouthWest,
}
