mod button;
mod checkbox;
mod radio;
mod slider;
mod text;

use ggez::graphics::{self, spritebatch::SpriteBatch, Image};
use ggez::Context;

pub struct Renderer<'a> {
    pub context: &'a mut Context,
    pub sprites: SpriteBatch,
    pub spritesheet: Image,
}

impl Renderer<'_> {
    pub fn new(context: &mut Context, spritesheet: Image) -> Renderer {
        Renderer {
            context,
            sprites: SpriteBatch::new(spritesheet.clone()),
            spritesheet,
        }
    }

    pub fn flush(&mut self) {
        graphics::draw(
            self.context,
            &self.sprites,
            graphics::DrawParam::default(),
        )
        .expect("Draw sprites");

        graphics::draw_queued_text(
            self.context,
            graphics::DrawParam::default(),
            Default::default(),
            graphics::FilterMode::Linear,
        )
        .expect("Draw text");
    }
}
