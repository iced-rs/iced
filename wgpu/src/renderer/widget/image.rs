use crate::{Primitive, Renderer};
use iced_native::{image, mouse, Layout};

impl image::Renderer for Renderer {
    fn dimensions(&self, handle: &image::Handle) -> (u32, u32) {
        self.image_pipeline.dimensions(handle)
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
