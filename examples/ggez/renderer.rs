mod button;
mod checkbox;
mod debugger;
mod image;
mod radio;
mod slider;
mod text;

use ggez::graphics::{
    self, spritebatch::SpriteBatch, Font, Image, MeshBuilder,
};
use ggez::Context;

pub struct Renderer<'a> {
    pub context: &'a mut Context,
    pub sprites: SpriteBatch,
    pub spritesheet: Image,
    pub font: Font,
    font_size: f32,
    debug_mesh: Option<MeshBuilder>,
}

impl Renderer<'_> {
    pub fn new(
        context: &mut Context,
        spritesheet: Image,
        font: Font,
    ) -> Renderer {
        Renderer {
            context,
            sprites: SpriteBatch::new(spritesheet.clone()),
            spritesheet,
            font,
            font_size: 20.0,
            debug_mesh: None,
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

        if let Some(debug_mesh) = self.debug_mesh.take() {
            let mesh =
                debug_mesh.build(self.context).expect("Build debug mesh");

            graphics::draw(self.context, &mesh, graphics::DrawParam::default())
                .expect("Draw debug mesh");
        }
    }
}
