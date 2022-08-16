use crate::image::atlas::Allocator;

#[derive(Debug)]
pub enum Layer {
    Empty,
    Busy(Allocator),
    Full,
}

impl Layer {
    pub fn is_empty(&self) -> bool {
        matches!(self, Layer::Empty)
    }
}
