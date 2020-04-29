use crate::canvas::{Event, Geometry, Size};
use iced_native::MouseCursor;

pub trait Program<Message> {
    fn update(&mut self, _event: Event, _bounds: Size) -> Option<Message> {
        None
    }

    fn draw(&self, bounds: Size) -> Vec<Geometry>;

    fn mouse_cursor(&self, _bounds: Size) -> MouseCursor {
        MouseCursor::default()
    }
}

impl<T, Message> Program<Message> for &mut T
where
    T: Program<Message>,
{
    fn update(&mut self, event: Event, bounds: Size) -> Option<Message> {
        T::update(self, event, bounds)
    }

    fn draw(&self, bounds: Size) -> Vec<Geometry> {
        T::draw(self, bounds)
    }

    fn mouse_cursor(&self, bounds: Size) -> MouseCursor {
        T::mouse_cursor(self, bounds)
    }
}
