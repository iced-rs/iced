use crate::{layout, Clipboard, Event, Hasher, Layer, Layout, Point, Size};
use std::rc::Rc;

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

    pub fn map<B>(self, f: Rc<dyn Fn(Message) -> B>) -> Overlay<'a, B, Renderer>
    where
        Message: 'static,
        Renderer: 'a,
        B: 'static,
    {
        Overlay {
            position: self.position,
            layer: Box::new(Map::new(self.layer, f)),
        }
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
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        self.layer.on_event(
            event,
            layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        )
    }
}

struct Map<'a, A, B, Renderer> {
    layer: Box<dyn Layer<A, Renderer> + 'a>,
    mapper: Rc<dyn Fn(A) -> B>,
}

impl<'a, A, B, Renderer> Map<'a, A, B, Renderer> {
    pub fn new(
        layer: Box<dyn Layer<A, Renderer> + 'a>,
        mapper: Rc<dyn Fn(A) -> B + 'static>,
    ) -> Map<'a, A, B, Renderer> {
        Map { layer, mapper }
    }
}

impl<'a, A, B, Renderer> Layer<B, Renderer> for Map<'a, A, B, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.layer.layout(renderer, bounds, position)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<B>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        let mut original_messages = Vec::new();

        self.layer.on_event(
            event,
            layout,
            cursor_position,
            &mut original_messages,
            renderer,
            clipboard,
        );

        original_messages
            .drain(..)
            .for_each(|message| messages.push((self.mapper)(message)));
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self.layer.draw(renderer, defaults, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        self.layer.hash_layout(state, position);
    }
}
