use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::image;
use iced_native::mouse;
use iced_native::Layout;

pub use iced_native::image::{Handle, Image};

impl<B> image::Renderer for Renderer<B>
where
    B: Backend + backend::Image,
{
    fn dimensions(&self, handle: &image::Handle) -> (u32, u32) {
        self.backend().dimensions(handle)
    }

    fn draw(
        &mut self,
        handle: image::Handle,
        layout: Layout<'_>,
    ) -> Self::Output {
        (
            Primitive::Image {
                handle,
                bounds: layout.bounds(),
            },
            mouse::Interaction::default(),
        )
    }
}
