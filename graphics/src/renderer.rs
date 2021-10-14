use crate::{Backend, Defaults, Primitive};
use iced_native::layout;
use iced_native::{Element, Rectangle};

/// A backend-agnostic renderer that supports all the built-in widgets.
#[derive(Debug)]
pub struct Renderer<B: Backend> {
    backend: B,
    primitive: Primitive,
}

impl<B: Backend> Renderer<B> {
    /// Creates a new [`Renderer`] from the given [`Backend`].
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            primitive: Primitive::None,
        }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn present(&mut self, f: impl FnOnce(&mut B, &Primitive)) {
        f(&mut self.backend, &self.primitive);
    }
}

impl<B> iced_native::Renderer for Renderer<B>
where
    B: Backend,
{
    type Defaults = Defaults;

    fn layout<'a, Message>(
        &mut self,
        element: &Element<'a, Message, Self>,
        limits: &layout::Limits,
    ) -> layout::Node {
        let layout = element.layout(self, limits);

        self.backend.trim_measurements();

        layout
    }

    fn with_layer(&mut self, _bounds: Rectangle, _f: impl FnOnce(&mut Self)) {}
}
