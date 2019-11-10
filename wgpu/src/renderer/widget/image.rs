use crate::{Primitive, Renderer};
use iced_native::{image, layout, Image, Layout, Length, MouseCursor, Size};

impl image::Renderer for Renderer {
    fn layout(&self, image: &Image, limits: &layout::Limits) -> layout::Node {
        let (width, height) = self.image_pipeline.dimensions(&image.path);

        let aspect_ratio = width as f32 / height as f32;

        // TODO: Deal with additional cases
        let (width, height) = match (image.width, image.height) {
            (Length::Units(width), _) => (
                image.width,
                Length::Units((width as f32 / aspect_ratio).round() as u16),
            ),
            (_, _) => {
                (Length::Units(width as u16), Length::Units(height as u16))
            }
        };

        let mut size = limits.width(width).height(height).resolve(Size::ZERO);

        size.height = size.width / aspect_ratio;

        layout::Node::new(size)
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
