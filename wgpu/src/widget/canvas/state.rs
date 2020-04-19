use crate::canvas::{Event, Geometry, Size};

pub trait State {
    fn update(&mut self, _event: Event, _bounds: Size) {}

    fn draw(&self, bounds: Size) -> Vec<Geometry>;
}

impl<T> State for &mut T
where
    T: State,
{
    fn update(&mut self, event: Event, bounds: Size) {
        T::update(self, event, bounds);
    }

    fn draw(&self, bounds: Size) -> Vec<Geometry> {
        T::draw(self, bounds)
    }
}
