use crate::image::atlas::Allocator;

#[derive(Debug)]
pub enum Layer {
    Empty,
    Busy(Allocator),
    Full,
}

impl Layer {
    pub fn is_empty(&self) -> bool {
        match self {
            Layer::Empty => true,
            _ => false,
        }
    }
}
