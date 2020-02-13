mod cached;

pub use cached::Cached;

use crate::{canvas::Frame, triangle};

use iced_native::Size;
use std::sync::Arc;

pub trait Layer: std::fmt::Debug {
    fn draw(&self, bounds: Size) -> Arc<triangle::Mesh2D>;
}

pub trait Drawable {
    fn draw(&self, frame: &mut Frame);
}
