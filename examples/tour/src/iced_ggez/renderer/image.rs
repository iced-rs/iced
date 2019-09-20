use super::Renderer;

use ggez::{graphics, nalgebra};
use iced_native::{image, Image, Layout, Length, Style};

pub struct Cache {
    images: std::collections::HashMap<String, graphics::Image>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            images: std::collections::HashMap::new(),
        }
    }

    fn get<'a>(
        &mut self,
        name: &'a str,
        context: &mut ggez::Context,
    ) -> graphics::Image {
        if let Some(image) = self.images.get(name) {
            return image.clone();
        }

        let mut image = graphics::Image::new(context, &format!("/{}", name))
            .expect("Load ferris image");

        image.set_filter(graphics::FilterMode::Linear);

        self.images.insert(name.to_string(), image.clone());

        image
    }
}

impl<'a> image::Renderer<&'a str> for Renderer<'_> {
    fn node(&mut self, image: &Image<&'a str>) -> iced_native::Node {
        let ggez_image = self.images.get(image.handle, self.context);

        let aspect_ratio =
            ggez_image.width() as f32 / ggez_image.height() as f32;

        let mut style = Style::default().align_self(image.align_self);

        style = match (image.width, image.height) {
            (Length::Units(width), _) => style.width(image.width).height(
                Length::Units((width as f32 / aspect_ratio).round() as u16),
            ),
            (_, _) => style
                .width(Length::Units(ggez_image.width()))
                .height(Length::Units(ggez_image.height())),
        };

        iced_native::Node::new(style)
    }

    fn draw(&mut self, image: &Image<&'a str>, layout: Layout<'_>) {
        let image = self.images.get(image.handle, self.context);
        let bounds = layout.bounds();

        // We should probably use batches to draw images efficiently and keep
        // draw side-effect free, but this is good enough for the example.
        graphics::draw(
            self.context,
            &image,
            graphics::DrawParam::new()
                .dest(nalgebra::Point2::new(bounds.x, bounds.y))
                .scale(nalgebra::Vector2::new(
                    bounds.width / image.width() as f32,
                    bounds.height / image.height() as f32,
                )),
        )
        .expect("Draw image");
    }
}
