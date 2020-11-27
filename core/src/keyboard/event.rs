use super::{KeyCode, Modifiers};

/// A keyboard event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/hecrj/iced/issues
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A keyboard key was pressed.
    KeyPressed {
        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: Modifiers,
    },

    /// A keyboard key was released.
    KeyReleased {
        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: Modifiers,
    },

    /// A unicode character was received.
    CharacterReceived(char),

    /// The keyboard modifiers have changed.
    ModifiersChanged(Modifiers),
}
