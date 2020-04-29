/// The state of the mouse cursor.
#[derive(Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
pub enum MouseCursor {
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

    /// The cursor is resizing a widget horizontally.
    ResizingHorizontally,

    /// The cursor is resizing a widget vertically.
    ResizingVertically,
}

impl Default for MouseCursor {
    fn default() -> MouseCursor {
        MouseCursor::Idle
    }
}
