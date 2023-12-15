use super::{KeyCode, Modifiers};

/// A keyboard event.
///
/// _**Note:** This type is largely incomplete! If you need to track
/// additional events, feel free to [open an issue] and share your use case!_
///
/// [open an issue]: https://github.com/iced-rs/iced/issues
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A keyboard key was pressed.
    KeyPressed {
        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: Modifiers,

        /// The text produced by the key press, if any.
        text: Option<String>,
    },

    /// A keyboard key was released.
    KeyReleased {
        /// The key identifier
        key_code: KeyCode,

        /// The state of the modifier keys
        modifiers: Modifiers,
    },

    /// The keyboard modifiers have changed.
    ModifiersChanged(Modifiers),
}
