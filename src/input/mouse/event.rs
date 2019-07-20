use super::Button;
use crate::input::ButtonState;

/// A mouse event.
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
    WheelScrolled {
        /// The number of horizontal lines scrolled
        delta_x: f32,

        /// The number of vertical lines scrolled
        delta_y: f32,
    },
}
