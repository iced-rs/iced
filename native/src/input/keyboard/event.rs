use super::{KeyCode, ModifiersState};
use crate::input::ButtonState;

/// A keyboard event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/hecrj/iced/issues
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A keyboard key was pressed or released.
    Input {
        /// The state of the key
        state: ButtonState,

        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: ModifiersState,
    },

    /// A unicode character was received.
    CharacterReceived(char),
}
