use crate::{Primitive, Renderer};
use iced_native::{image, layout, Image, Layout, MouseCursor, Rectangle};

impl image::Renderer for Renderer {
    fn layout(&self, image: &Image, limits: &layout::Limits) -> Layout {
        // TODO
        Layout::new(Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        })
    }

    fn draw(&mut self, image: &Image, layout: &Layout) -> Self::Output {
        (
            Primitive::Image {
                path: image.path.clone(),
                bounds: layout.bounds(),
            },
            MouseCursor::OutOfBounds,
        )
    }
}
