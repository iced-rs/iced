use super::Button;
use crate::input::ButtonState;

/// A mouse event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/hecrj/iced/issues
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// The mouse cursor entered the window.
    CursorEntered,

    /// The mouse cursor left the window.
    CursorLeft,

    /// The mouse cursor was moved
    CursorMoved {
        /// The X coordinate of the mouse position
        x: f32,

        /// The Y coordinate of the mouse position
        y: f32,
    },

    /// A mouse button was pressed or released.
    Input {
        /// The state of the button
        state: ButtonState,

        /// The button identifier
        button: Button,
    },

    /// The mouse wheel was scrolled.
    WheelScrolled { delta: ScrollDelta },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDelta {
    Lines {
        /// The number of horizontal lines scrolled
        x: f32,

        /// The number of vertical lines scrolled
        y: f32,
    },
    Pixels {
        /// The number of horizontal pixels scrolled
        x: f32,
        /// The number of vertical pixels scrolled
        y: f32,
    },
}
