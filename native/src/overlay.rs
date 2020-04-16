use crate::{layout, Clipboard, Event, Hasher, Layer, Layout, Point, Size};

#[allow(missing_debug_implementations)]
pub struct Overlay<'a, Message, Renderer> {
    position: Point,
    layer: Box<dyn Layer<Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> Overlay<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    pub fn new(
        position: Point,
        layer: Box<dyn Layer<Message, Renderer> + 'a>,
    ) -> Self {
        Self { position, layer }
    }

    pub fn layout(&self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.layer.layout(renderer, bounds, self.position)
    }

    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self.layer.draw(renderer, defaults, layout, cursor_position)
    }

    pub fn hash_layout(&self, state: &mut Hasher) {
        self.layer.hash_layout(state, self.position);
    }

    pub fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer,
        _clipboard: Option<&dyn Clipboard>,
    ) {
    }
}
