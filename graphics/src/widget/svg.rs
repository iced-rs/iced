//! Display vector graphics in your application.
use crate::backend::{self, Backend};
use crate::{Primitive, Rectangle, Renderer};
use iced_native::widget::svg;

pub use iced_native::widget::svg::{Handle, Svg};

impl<B> svg::Renderer for Renderer<B>
where
    B: Backend + backend::Svg,
{
    fn dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
        self.backend().viewport_dimensions(handle)
    }

    fn draw(&mut self, handle: svg::Handle, bounds: Rectangle) {
        self.draw_primitive(Primitive::Svg { handle, bounds })
    }
}
