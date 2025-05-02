pub use crate::Overlay;

use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::{Clipboard, Event, Layout, Shell, Size};

/// A generic [`Overlay`].
#[allow(missing_debug_implementations)]
pub struct Element<'a, Message, Theme, Renderer> {
    overlay: Box<dyn Overlay<Message, Theme, Renderer> + 'a>,
}

impl<'a, Message, Theme, Renderer> Element<'a, Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Creates a new [`Element`] containing the given [`Overlay`].
    pub fn new(
        overlay: Box<dyn Overlay<Message, Theme, Renderer> + 'a>,
    ) -> Self {
        Self { overlay }
    }

    /// Returns a reference to the [`Overlay`] of the [`Element`],
    pub fn as_overlay(&self) -> &dyn Overlay<Message, Theme, Renderer> {
        self.overlay.as_ref()
    }

    /// Returns a mutable reference to the [`Overlay`] of the [`Element`],
    pub fn as_overlay_mut(
        &mut self,
    ) -> &mut dyn Overlay<Message, Theme, Renderer> {
        self.overlay.as_mut()
    }

    /// Applies a transformation to the produced message of the [`Element`].
    pub fn map<B>(
        self,
        f: &'a dyn Fn(Message) -> B,
    ) -> Element<'a, B, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: 'a,
        B: 'a,
    {
        Element {
            overlay: Box::new(Map::new(self.overlay, f)),
        }
    }
}

struct Map<'a, A, B, Theme, Renderer> {
    content: Box<dyn Overlay<A, Theme, Renderer> + 'a>,
    mapper: &'a dyn Fn(A) -> B,
}

impl<'a, A, B, Theme, Renderer> Map<'a, A, B, Theme, Renderer> {
    pub fn new(
        content: Box<dyn Overlay<A, Theme, Renderer> + 'a>,
        mapper: &'a dyn Fn(A) -> B,
    ) -> Map<'a, A, B, Theme, Renderer> {
        Map { content, mapper }
    }
}

impl<A, B, Theme, Renderer> Overlay<B, Theme, Renderer>
    for Map<'_, A, B, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.content.layout(renderer, bounds)
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.operate(layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
    ) {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        self.content.update(
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
        );

        shell.merge(local_shell, self.mapper);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.mouse_interaction(layout, cursor, renderer)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.draw(renderer, theme, style, layout, cursor);
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<Element<'a, B, Theme, Renderer>> {
        self.content
            .overlay(layout, renderer)
            .map(|overlay| overlay.map(self.mapper))
    }
}
