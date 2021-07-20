use crate::keyboard::{KeyCode, Modifiers};

/// Representation of a hotkey, consists on the combination of a [`KeyCode`] and [`Modifiers`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hotkey {
    /// The key that represents this hotkey.
    pub key: KeyCode,

    /// The list of modifiers that represents this hotkey.
    pub modifiers: Modifiers,
}

impl Hotkey {
    /// Creates a new [`Hotkey`] with the given [`Modifiers`] and [`KeyCode`].
    pub fn new(modifiers: Modifiers, key: KeyCode) -> Self {
        Self { modifiers, key }
    }
}
