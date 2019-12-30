use crate::{Primitive, Renderer};
use iced_native::{empty, MouseCursor, Rectangle};

impl empty::Renderer for Renderer {
    fn draw(&mut self, _bounds: Rectangle) -> Self::Output {
        (Primitive::None, MouseCursor::OutOfBounds)
    }
}
