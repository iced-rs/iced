mod widget;

use crate::{Backend, Defaults, Primitive};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::{Background, Color, Element, Point, Widget};

pub struct Renderer<B> {
    backend: B,
}

impl<B> Renderer<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub fn backend(&self) -> &B {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

impl<B> iced_native::Renderer for Renderer<B>
where
    B: Backend,
{
    type Output = (Primitive, mouse::Interaction);
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
}

impl<B> layout::Debugger for Renderer<B>
where
    B: Backend,
{
    fn explain<Message>(
        &mut self,
        defaults: &Defaults,
        widget: &dyn Widget<Message, Self>,
        layout: Layout<'_>,
        cursor_position: Point,
        color: Color,
    ) -> Self::Output {
        let (primitive, cursor) =
            widget.draw(self, defaults, layout, cursor_position);

        let mut primitives = Vec::new();

        explain_layout(layout, color, &mut primitives);
        primitives.push(primitive);

        (Primitive::Group { primitives }, cursor)
    }
}

fn explain_layout(
    layout: Layout<'_>,
    color: Color,
    primitives: &mut Vec<Primitive>,
) {
    primitives.push(Primitive::Quad {
        bounds: layout.bounds(),
        background: Background::Color(Color::TRANSPARENT),
        border_radius: 0,
        border_width: 1,
        border_color: [0.6, 0.6, 0.6, 0.5].into(),
    });

    for child in layout.children() {
        explain_layout(child, color, primitives);
    }
}
