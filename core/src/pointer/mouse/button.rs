/// The button of a mouse.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    /// The left mouse button.
    Left,

    /// The right mouse button.
    Right,

    /// The middle (wheel) button.
    Middle,

    /// The back mouse button.
    Back,

    /// The forward mouse button.
    Forward,

    /// Some other button.
    Other(u16),
}
