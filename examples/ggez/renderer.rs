mod text;

use ggez::graphics::{self, Color};
use ggez::Context;

pub struct Renderer<'a> {
    pub context: &'a mut Context,
}

impl Renderer<'_> {
    pub fn flush(&mut self) {
        graphics::draw_queued_text(
            self.context,
            graphics::DrawParam::default(),
            Default::default(),
            graphics::FilterMode::Linear,
        )
        .expect("Draw text");
    }
}

impl iced::Renderer for Renderer<'_> {
    type Color = Color;

    fn explain(&mut self, layout: &iced::Layout<'_>, color: Color) {}
}
