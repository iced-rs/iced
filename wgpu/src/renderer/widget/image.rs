use crate::{Primitive, Renderer};
use iced_native::{image, Layout, MouseCursor};

impl image::Renderer for Renderer {
    fn dimensions(&self, path: &str) -> (u32, u32) {
        self.image_pipeline.dimensions(path)
    }

    fn draw(&mut self, path: &str, layout: Layout<'_>) -> Self::Output {
        (
            Primitive::Image {
                path: String::from(path),
                bounds: layout.bounds(),
            },
            MouseCursor::OutOfBounds,
        )
    }
}
