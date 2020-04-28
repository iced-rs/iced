use crate::canvas::{Event, Geometry, Size};

pub trait Program<Message> {
    fn update(&mut self, _event: Event, _bounds: Size) -> Option<Message> {
        None
    }

    fn draw(&self, bounds: Size) -> Vec<Geometry>;
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
}
