use crate::{Primitive, Renderer};
use iced_native::{image, Image, Layout, MouseCursor};

impl image::Renderer for Renderer {
    fn dimensions(&self, path: &str) -> (u32, u32) {
        self.image_pipeline.dimensions(path)
    }

    fn draw(&mut self, image: &Image, layout: Layout<'_>) -> Self::Output {
        (
            Primitive::Image {
                path: image.path.clone(),
                bounds: layout.bounds(),
            },
            MouseCursor::OutOfBounds,
        )
    }
}
