use crate::{Point, keyboard::Modifiers};

use super::Button;

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
        /// the modifier keys at the time of the cursor movement
        modifiers: Modifiers,
    },

    /// A mouse button was pressed.
    ButtonPressed {
        /// the button that was pressed
        button: Button,
        /// the modifier keys at the time of the button press
        modifiers: Modifiers,
    },

    /// A mouse button was released.
    ButtonReleased {
        /// the button that was released
        button: Button,
        /// the modifier keys at the time of the button release
        modifiers: Modifiers,
    },

    /// The mouse wheel was scrolled.
    WheelScrolled {
        /// The scroll movement.
        delta: ScrollDelta,
        /// the modifier keys at the time of the scroll movement
        modifiers: Modifiers,
    },
}

/// A scroll movement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollDelta {
    /// A line-based scroll movement
    Lines {
        /// The number of horizontal lines scrolled
        x: f32,

        /// The number of vertical lines scrolled
        y: f32,
    },
    /// A pixel-based scroll movement
    Pixels {
        /// The number of horizontal pixels scrolled
        x: f32,
        /// The number of vertical pixels scrolled
        y: f32,
    },
}
