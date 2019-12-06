use crate::{svg::Handle, Primitive, Renderer};
use iced_native::{
    icon, MouseCursor, Rectangle,
};
use std::path::Path;

impl icon::Renderer for Renderer {
    fn draw(
        &mut self,
        bounds: Rectangle,
        path: &Path,
    ) -> Self::Output {
        (
            Primitive::Svg {
                handle: Handle::from_path(path),
                bounds,
            },
            MouseCursor::OutOfBounds,
        )
    }
}