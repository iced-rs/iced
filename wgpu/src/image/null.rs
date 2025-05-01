pub use crate::graphics::Image;

#[derive(Debug, Default)]
pub struct Batch;

impl Batch {
    pub fn push(&mut self, _image: Image) {}

    pub fn clear(&mut self) {}

    pub fn is_empty(&self) -> bool {
        true
    }
}
