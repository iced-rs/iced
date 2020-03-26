use crate::{Primitive, Renderer};
use iced_native::{svg, Depth, Layout, MouseCursor};

impl svg::Renderer for Renderer {
    fn dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
        self.image_pipeline.viewport_dimensions(handle)
    }

    fn draw(
        &mut self,
        handle: svg::Handle,
        layout: Layout<'_>,
    ) -> Self::Output {
        (
            (
                Primitive::Svg {
                    handle,
                    bounds: layout.bounds(),
                },
                Depth::None,
            ),
            MouseCursor::OutOfBounds,
        )
    }
}
