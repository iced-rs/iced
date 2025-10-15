use crate::SmolStr;
use crate::keyboard::key;
use crate::keyboard::{Key, Location, Modifiers};

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
        /// The key pressed. Tries to meet expectations, currently equal to `modified_key`.
        key: Key,

        /// The key pressed with all keyboard modifiers applied, except Ctrl.
        modified_key: Key,

        /// The key on keyboard layer 0 pressed.
        baselayer_key: Key,

        /// The physical key pressed.
        physical_key: key::Physical,

        /// The location of the key.
        location: Location,

        /// The state of the modifier keys.
        modifiers: Modifiers,

        /// The text produced by the key press, if any.
        text: Option<SmolStr>,
    },

    /// A keyboard key was released.
    KeyReleased {
        /// The key released. Tries to meet expectations, currently equal to `modified_key`.
        key: Key,

        /// The key released with all keyboard modifiers applied, except Ctrl.
        modified_key: Key,

        /// The key on keyboard layer 0 released.
        baselayer_key: Key,

        /// The physical key released.
        physical_key: key::Physical,

        /// The location of the key.
        location: Location,

        /// The state of the modifier keys.
        modifiers: Modifiers,
    },

    /// The keyboard modifiers have changed.
    ModifiersChanged(Modifiers),
}
