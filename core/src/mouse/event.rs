use crate::{Point, Vector};

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
    },

    /// The mouse was moved.
    ///
    /// This will fire in situations where [`CursorMoved`] might not,
    /// such as the mouse being outside of the window or hitting the edge
    /// of the monitor, and can be used to get the correct motion when
    /// [`CursorGrab`] is set to something other than [`None`].
    ///
    /// [`CursorMoved`]: Event::CursorMoved
    /// [`CursorGrab`]: super::super::window::CursorGrab
    /// [`None`]: super::super::window::CursorGrab::None
    MouseMotion {
        /// The change in position of the mouse cursor
        delta: Vector,
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
