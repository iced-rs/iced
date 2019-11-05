/// The state of the mouse cursor.
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
pub enum MouseCursor {
    /// The cursor is out of the bounds of the user interface.
    OutOfBounds,

    /// The cursor is over a non-interactive widget.
    Idle,

    /// The cursor is over a clickable widget.
    Pointer,

    /// The cursor is over a busy widget.
    Working,

    /// The cursor is over a grabbable widget.
    Grab,

    /// The cursor is grabbing a widget.
    Grabbing,

    /// The cursor is over a text widget.
    Text,
}
