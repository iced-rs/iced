use crate::input::{keyboard, mouse};

/// A user interface event.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Event {
    /// A keyboard event
    Keyboard(keyboard::Event),

    /// A mouse event
    Mouse(mouse::Event),
}
