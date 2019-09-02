/// The button of a mouse.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Button {
    /// The left mouse button.
    Left,

    /// The right mouse button.
    Right,

    /// The middle (wheel) button.
    Middle,

    /// Some other button.
    Other(u8),
}

#[cfg(feature = "winit")]
impl From<winit::event::MouseButton> for super::Button {
    fn from(mouse_button: winit::event::MouseButton) -> Self {
        match mouse_button {
            winit::event::MouseButton::Left => Button::Left,
            winit::event::MouseButton::Right => Button::Right,
            winit::event::MouseButton::Middle => Button::Middle,
            winit::event::MouseButton::Other(other) => Button::Other(other),
        }
    }
}
