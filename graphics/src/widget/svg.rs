//! Display vector graphics in your application.
use crate::backend::{self, Backend};
use crate::Renderer;
use iced_native::svg;

pub use iced_native::svg::{Handle, Svg};

impl<B> svg::Renderer for Renderer<B>
where
    B: Backend + backend::Svg,
{
    fn dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
        self.backend().viewport_dimensions(handle)
    }
}
