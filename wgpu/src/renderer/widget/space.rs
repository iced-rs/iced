use crate::{Primitive, Renderer};
use iced_native::{mouse, space, Rectangle};

impl space::Renderer for Renderer {
    fn draw(&mut self, _bounds: Rectangle) -> Self::Output {
        (Primitive::None, mouse::Interaction::default())
    }
}
