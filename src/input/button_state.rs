/// The state of a button.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum ButtonState {
    /// The button is pressed.
    Pressed,

    /// The button is __not__ pressed.
    Released,
}

#[cfg(feature = "winit")]
impl From<winit::event::ElementState> for ButtonState {
    fn from(element_state: winit::event::ElementState) -> Self {
        match element_state {
            winit::event::ElementState::Pressed => ButtonState::Pressed,
            winit::event::ElementState::Released => ButtonState::Released,
        }
    }
}
