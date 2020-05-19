use crate::{Backend, Primitive, Renderer};
use iced_native::mouse;
use iced_native::space;
use iced_native::Rectangle;

pub use iced_native::Space;

impl<B> space::Renderer for Renderer<B>
where
    B: Backend,
{
    fn draw(&mut self, _bounds: Rectangle) -> Self::Output {
        (Primitive::None, mouse::Interaction::default())
    }
}
