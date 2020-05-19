use crate::backend::{self, Backend};
use crate::{Primitive, Renderer};
use iced_native::{mouse, svg, Layout};

pub use iced_native::svg::{Handle, Svg};

impl<B> svg::Renderer for Renderer<B>
where
    B: Backend + backend::Svg,
{
    fn dimensions(&self, handle: &svg::Handle) -> (u32, u32) {
        self.backend().viewport_dimensions(handle)
    }

    fn draw(
        &mut self,
        handle: svg::Handle,
        layout: Layout<'_>,
    ) -> Self::Output {
        (
            Primitive::Svg {
                handle,
                bounds: layout.bounds(),
            },
            mouse::Interaction::default(),
        )
    }
}
