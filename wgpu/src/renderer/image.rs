use crate::{Primitive, Renderer};
use iced_native::{image, Image, Layout, Length, MouseCursor, Node, Style};

impl image::Renderer for Renderer {
    fn node(&self, image: &Image) -> Node {
        let (width, height) = self.image_pipeline.dimensions(&image.path);

        let aspect_ratio = width as f32 / height as f32;

        let mut style = Style::default().align_self(image.align_self);

        // TODO: Deal with additional cases
        style = match (image.width, image.height) {
            (Length::Units(width), _) => style.width(image.width).height(
                Length::Units((width as f32 / aspect_ratio).round() as u16),
            ),
            (_, _) => style
                .width(Length::Units(width as u16))
                .height(Length::Units(height as u16)),
        };

        Node::new(style)
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
