/// The location of a key on the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Location {
    /// The standard group of keys on the keyboard.
    Standard,
    /// The left side of the keyboard.
    Left,
    /// The right side of the keyboard.
    Right,
    /// The numpad of the keyboard.
    Numpad,
}
