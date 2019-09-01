/// The state of a button.
#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum ButtonState {
    /// The button is pressed.
    Pressed,

    /// The button is __not__ pressed.
    Released,
}

#[cfg(feature = "winit")]
mod winit_conversion {
    use winit::event::ElementState;

    impl From<ElementState> for super::ButtonState {
        fn from(element_state: ElementState) -> Self {
            match element_state {
                ElementState::Pressed => super::ButtonState::Pressed,
                ElementState::Released => super::ButtonState::Released,
            }
        }
    }
}
