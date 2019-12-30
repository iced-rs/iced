use crate::{Primitive, Renderer};
use iced_native::{space, MouseCursor, Rectangle};

impl space::Renderer for Renderer {
    fn draw(&mut self, _bounds: Rectangle) -> Self::Output {
        (Primitive::None, MouseCursor::OutOfBounds)
    }
}
