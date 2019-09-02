/// The state of the mouse cursor.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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
}

#[cfg(feature = "winit")]
impl From<MouseCursor> for winit::window::CursorIcon {
    fn from(mouse_cursor: MouseCursor) -> winit::window::CursorIcon {
        match mouse_cursor {
            MouseCursor::OutOfBounds => winit::window::CursorIcon::Default,
            MouseCursor::Idle => winit::window::CursorIcon::Default,
            MouseCursor::Pointer => winit::window::CursorIcon::Hand,
            MouseCursor::Working => winit::window::CursorIcon::Progress,
            MouseCursor::Grab => winit::window::CursorIcon::Grab,
            MouseCursor::Grabbing => winit::window::CursorIcon::Grabbing,
        }
    }
}
