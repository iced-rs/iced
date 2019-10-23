/// The state of a button.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum ButtonState {
    /// The button is pressed.
    Pressed,

    /// The button is __not__ pressed.
    Released,
}
