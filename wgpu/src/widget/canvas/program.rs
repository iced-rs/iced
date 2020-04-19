use crate::canvas::{Event, Layer, Size};

pub trait Program {
    type Input;

    fn layers<'a>(&'a self, input: &'a Self::Input)
        -> Vec<Box<dyn Layer + 'a>>;

    fn update<'a>(
        &'a mut self,
        _event: Event,
        _bounds: Size,
        _input: &'a Self::Input,
    ) {
    }
}
