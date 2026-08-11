use crate::Point;

use super::{Button, ScrollDelta};

/// A mouse event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// The mouse cursor entered the window.
    CursorEntered,

    /// The mouse cursor left the window.
    CursorLeft,

    /// The mouse cursor was moved
    CursorMoved {
        /// The new position of the mouse cursor
        position: Point,
    },

    /// A mouse button was pressed.
    ButtonPressed(Button),

    /// A mouse button was released.
    ButtonReleased(Button),

    /// The mouse wheel was scrolled.
    WheelScrolled {
        /// The scroll movement.
        delta: ScrollDelta,
    },
}
