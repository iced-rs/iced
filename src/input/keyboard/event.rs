use super::KeyCode;
use crate::input::ButtonState;

#[derive(Debug, Clone, Copy, PartialEq)]
/// A keyboard event.
pub enum Event {
    /// A keyboard key was pressed or released.
    Input {
        /// The state of the key
        state: ButtonState,

        /// The key identifier
        key_code: KeyCode,
    },

    /// Text was entered.
    TextEntered {
        /// The character entered
        character: char,
    },
}
