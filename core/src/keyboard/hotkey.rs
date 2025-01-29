use crate::keyboard::key;
use crate::keyboard::{Key, Location, Modifiers};

/// A specific keyboard combination meant to trigger some action.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Hotkey {
    /// The keyboard key.
    pub key: Key,

    /// The keyboard modifiers.
    ///
    /// By default, hotkeys are triggered with [`Modifiers::COMMAND`].
    pub modifiers: Modifiers,

    /// The keyboard location, if relevant.
    ///
    /// A `None` value will trigger the hotkey independently
    /// of the actual keyboard location pressed by the user.
    pub location: Option<Location>,
}

impl From<char> for Hotkey {
    fn from(c: char) -> Self {
        Self::from(Key::from(c))
    }
}

impl From<&str> for Hotkey {
    fn from(s: &str) -> Self {
        Self::from(Key::from(s))
    }
}

impl From<key::Named> for Hotkey {
    fn from(key: key::Named) -> Self {
        Self::from(Key::Named(key))
    }
}

impl From<Key> for Hotkey {
    fn from(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::COMMAND,
            location: None,
        }
    }
}

impl From<(Modifiers, Key)> for Hotkey {
    fn from((modifiers, key): (Modifiers, Key)) -> Self {
        Self {
            key,
            modifiers,
            location: None,
        }
    }
}
