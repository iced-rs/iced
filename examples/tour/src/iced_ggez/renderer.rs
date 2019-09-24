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

pub use image::Cache;

pub struct Renderer<'a> {
    pub context: &'a mut Context,
    pub images: &'a mut image::Cache,
    pub sprites: SpriteBatch,
    pub spritesheet: Image,
    pub font: Font,
    font_size: f32,
    debug_mesh: Option<MeshBuilder>,
}

impl<'a> Renderer<'a> {
    pub fn new(
        context: &'a mut Context,
        images: &'a mut image::Cache,
        spritesheet: Image,
        font: Font,
    ) -> Renderer<'a> {
        Renderer {
            context,
            images,
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

pub fn into_color(color: iced_native::Color) -> graphics::Color {
    graphics::Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: color.a,
    }
}
