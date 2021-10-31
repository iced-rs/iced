//! Display images in your user interface.
pub mod viewer;

use crate::backend::{self, Backend};
use crate::{Primitive, Rectangle, Renderer};

use iced_native::widget::image;

pub use iced_native::widget::image::{Handle, Image, Viewer};

impl<B> image::Renderer for Renderer<B>
where
    B: Backend + backend::Image,
{
    fn dimensions(&self, handle: &image::Handle) -> (u32, u32) {
        self.backend().dimensions(handle)
    }

    fn draw(&mut self, handle: image::Handle, bounds: Rectangle) {
        self.draw_primitive(Primitive::Image { handle, bounds })
    }
}
