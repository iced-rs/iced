use crate::Point;

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

    /// A mouse button was pressed.
    ButtonPressed(Button),

    /// A mouse button was released.
    ButtonReleased(Button),

    /// The mouse wheel or touchpad was scrolled
    WheelScrolled {
        /// The scroll movement.
        delta: ScrollDelta,
        /// The scroll phase (for mouse - only Move)
        phase: ScrollPhase,
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

/// A scrolling phase (for mouses it would be only Moved)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollPhase {
    /// Start scrolling with touchpad
    Started,
    /// Scrolling with mouse or continue scrolling with touchpad
    Moved,
    /// End of scrolling with touchpad
    Ended,
    /// System cancel process of srcolling (window lost focus, compositor interruption etc)
    Cancelled,
}
