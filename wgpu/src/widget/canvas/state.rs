use crate::canvas::{Event, Geometry, Size};

pub trait State<Message> {
    fn update(&mut self, _event: Event, _bounds: Size) -> Option<Message> {
        None
    }

    fn draw(&self, bounds: Size) -> Vec<Geometry>;
}

impl<T, Message> State<Message> for &mut T
where
    T: State<Message>,
{
    fn update(&mut self, event: Event, bounds: Size) -> Option<Message> {
        T::update(self, event, bounds)
    }

    fn draw(&self, bounds: Size) -> Vec<Geometry> {
        T::draw(self, bounds)
    }
}
