use super::KeyCode;
use crate::input::ButtonState;

#[derive(Debug, Clone, Copy, PartialEq)]
/// A keyboard event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/hecrj/iced/issues
pub enum Event {
    /// A keyboard key was pressed or released.
    Input {
        /// The state of the key
        state: ButtonState,

        /// The key identifier
        key_code: KeyCode,
    },

    /// A unicode character was received.
    CharacterReceived(char),
}
