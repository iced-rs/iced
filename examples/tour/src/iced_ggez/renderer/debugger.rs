use super::{into_color, Renderer};
use ggez::graphics::{DrawMode, MeshBuilder, Rect};

impl iced::renderer::Debugger for Renderer<'_> {
    type Color = iced::Color;

    fn explain(&mut self, layout: &iced::Layout<'_>, color: iced::Color) {
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
            into_color(color),
        );

        self.debug_mesh = Some(debug_mesh);

        for child in layout.children() {
            self.explain(&child, color);
        }
    }
}
