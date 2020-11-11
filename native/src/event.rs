//! Handle events of a user interface.
use crate::{keyboard, mouse, window};

/// A user interface event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/hecrj/iced/issues
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// A keyboard event
    Keyboard(keyboard::Event),

    /// A mouse event
    Mouse(mouse::Event),

    /// A window event
    Window(window::Event),
}

/// The status of an [`Event`] after being processed.
///
/// [`Event`]: enum.Event.html
/// [`UserInterface`]: ../struct.UserInterface.html
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Event`] was _NOT_ handled by any widget.
    ///
    /// [`Event`]: enum.Event.html
    Ignored,

    /// The [`Event`] was handled and processed by a widget.
    ///
    /// [`Event`]: enum.Event.html
    Captured,
}
