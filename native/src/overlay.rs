use crate::{layout, Clipboard, Event, Hasher, Layout, Point};

pub trait Overlay<Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output;

    fn hash_layout(&self, state: &mut Hasher);

    fn on_event(
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
