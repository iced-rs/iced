use super::Renderer;
use ggez::graphics::{Color, DrawMode, MeshBuilder, Rect};

impl iced::renderer::Debugger for Renderer<'_> {
    type Color = Color;

    fn explain(&mut self, layout: &iced::Layout<'_>, color: Color) {
        let bounds = layout.bounds();

        let mut debug_mesh =
            self.debug_mesh.take().unwrap_or(MeshBuilder::new());

        debug_mesh.rectangle(
            DrawMode::stroke(1.0),
            Rect {
                x: bounds.x,
                y: bounds.y,
                w: bounds.width,
                h: bounds.height,
            },
            color,
        );

        self.debug_mesh = Some(debug_mesh);

        for child in layout.children() {
            self.explain(&child, color);
        }
    }
}
