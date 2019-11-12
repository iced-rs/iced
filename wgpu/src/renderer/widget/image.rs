use iced_native::{Length, Size};

impl iced_native::image::Renderer for crate::Renderer {

    fn layout(&self, image: &iced_native::Image, limits: &iced_native::layout::Limits) -> iced_native::layout::Node {
        let (width, height) = self.image_pipeline.dimensions(&image.path).unwrap_or_else(|e| { println!("{} '{}'", e, image.path); (0,0) });

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

        iced_native::layout::Node::new(size)
    }

    fn draw(&mut self, image: &iced_native::Image, layout: iced_native::Layout<'_>) -> Self::Output {
        (
            crate::Primitive::Image {
                path: image.path.clone(),
                bounds: layout.bounds(),
            },
            iced_native::MouseCursor::OutOfBounds,
        )
    }
}
