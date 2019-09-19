use super::Renderer;

use ggez::{graphics, nalgebra};
use iced::image;

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
    fn node(
        &mut self,
        style: iced::Style,
        name: &&'a str,
        width: Option<u16>,
        height: Option<u16>,
        _source: Option<iced::Rectangle<u16>>,
    ) -> iced::Node {
        let image = self.images.get(name, self.context);

        let aspect_ratio = image.width() as f32 / image.height() as f32;

        let style = match (width, height) {
            (Some(width), Some(height)) => style.width(width).height(height),
            (Some(width), None) => style
                .width(width)
                .height((width as f32 / aspect_ratio).round() as u16),
            (None, Some(height)) => style
                .height(height)
                .width((height as f32 * aspect_ratio).round() as u16),
            (None, None) => style.width(image.width()).height(image.height()),
        };

        iced::Node::new(style)
    }

    fn draw(
        &mut self,
        name: &&'a str,
        bounds: iced::Rectangle,
        _source: Option<iced::Rectangle<u16>>,
    ) {
        let image = self.images.get(name, self.context);

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
