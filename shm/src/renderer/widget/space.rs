use crate::{Primitive, Renderer};
use iced_native::{space, mouse::Interaction, Rectangle};

impl space::Renderer for Renderer {
    fn draw(&mut self, _bounds: Rectangle) -> Self::Output {
        (Primitive::None, Interaction::Idle)
    }
}
