pub use crate::Overlay;

use crate::event::{self, Event};
use crate::layout;
use crate::{Clipboard, Hasher, Layout, Point, Size, Vector};

/// A generic [`Overlay`].
#[allow(missing_debug_implementations)]
pub struct Element<'a, Message, Renderer> {
    position: Point,
    overlay: Box<dyn Overlay<Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> Element<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Creates a new [`Element`] containing the given [`Overlay`].
    pub fn new(
        position: Point,
        overlay: Box<dyn Overlay<Message, Renderer> + 'a>,
    ) -> Self {
        Self { position, overlay }
    }

    /// Translates the [`Element`].
    pub fn translate(mut self, translation: Vector) -> Self {
        self.position = self.position + translation;
        self
    }

    /// Applies a transformation to the produced message of the [`Element`].
    pub fn map<B>(self, f: &'a dyn Fn(Message) -> B) -> Element<'a, B, Renderer>
    where
        Message: 'a,
        Renderer: 'a,
        B: 'static,
    {
        Element {
            position: self.position,
            overlay: Box::new(Map::new(self.overlay, f)),
        }
    }

    /// Computes the layout of the [`Element`] in the given bounds.
    pub fn layout(&self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.overlay.layout(renderer, bounds, self.position)
    }

    /// Processes a runtime [`Event`].
    pub fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        self.overlay.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            messages,
        )
    }

    /// Draws the [`Element`] and its children using the given [`Layout`].
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self.overlay
            .draw(renderer, defaults, layout, cursor_position)
    }

    /// Computes the _layout_ hash of the [`Element`].
    pub fn hash_layout(&self, state: &mut Hasher) {
        self.overlay.hash_layout(state, self.position);
    }
}

struct Map<'a, A, B, Renderer> {
    content: Box<dyn Overlay<A, Renderer> + 'a>,
    mapper: &'a dyn Fn(A) -> B,
}

impl<'a, A, B, Renderer> Map<'a, A, B, Renderer> {
    pub fn new(
        content: Box<dyn Overlay<A, Renderer> + 'a>,
        mapper: &'a dyn Fn(A) -> B,
    ) -> Map<'a, A, B, Renderer> {
        Map { content, mapper }
    }
}

impl<'a, A, B, Renderer> Overlay<B, Renderer> for Map<'a, A, B, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        self.content.layout(renderer, bounds, position)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<B>,
    ) -> event::Status {
        let mut original_messages = Vec::new();

        let event_status = self.content.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            &mut original_messages,
        );

        original_messages
            .drain(..)
            .for_each(|message| messages.push((self.mapper)(message)));

        event_status
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        self.content
            .draw(renderer, defaults, layout, cursor_position)
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        self.content.hash_layout(state, position);
    }
}
