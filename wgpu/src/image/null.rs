pub use crate::graphics::Image;

#[derive(Default)]
pub struct Batch;

impl Batch {
    pub fn push(&mut self, _image: Image) {}

    pub fn clear(&mut self) {}
}
