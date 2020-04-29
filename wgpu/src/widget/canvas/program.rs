use crate::canvas::{Cursor, Event, Geometry};
use iced_native::{MouseCursor, Rectangle};

pub trait Program<Message> {
    fn update(
        &mut self,
        _event: Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> Option<Message> {
        None
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry>;

    fn mouse_cursor(&self, _bounds: Rectangle, _cursor: Cursor) -> MouseCursor {
        MouseCursor::default()
    }
}

impl<T, Message> Program<Message> for &mut T
where
    T: Program<Message>,
{
    fn update(
        &mut self,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<Message> {
        T::update(self, event, bounds, cursor)
    }

    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        T::draw(self, bounds, cursor)
    }

    fn mouse_cursor(&self, bounds: Rectangle, cursor: Cursor) -> MouseCursor {
        T::mouse_cursor(self, bounds, cursor)
    }
}
