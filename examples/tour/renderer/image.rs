use super::Renderer;

use ggez::{graphics, nalgebra};
use iced::image;

impl image::Renderer<graphics::Image> for Renderer<'_> {
    fn node(
        &self,
        style: iced::Style,
        image: &graphics::Image,
        width: Option<u16>,
        height: Option<u16>,
        _source: Option<iced::Rectangle<u16>>,
    ) -> iced::Node {
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
        image: &graphics::Image,
        bounds: iced::Rectangle,
        _source: Option<iced::Rectangle<u16>>,
    ) {
        // We should probably use batches to draw images efficiently and keep
        // draw side-effect free, but this is good enough for the example.
        graphics::draw(
            self.context,
            image,
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
