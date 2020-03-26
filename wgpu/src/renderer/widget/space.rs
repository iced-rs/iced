use crate::{Primitive, Renderer};
use iced_native::{space, Depth, MouseCursor, Rectangle};

impl space::Renderer for Renderer {
    fn draw(&mut self, _bounds: Rectangle) -> Self::Output {
        ((Primitive::None, Depth::None), MouseCursor::OutOfBounds)
    }
}
