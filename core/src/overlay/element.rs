pub use crate::Overlay;

use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::{Clipboard, Layout, Point, Rectangle, Shell, Size};

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

    /// Computes the layout of the [`Element`] in the given bounds.
    pub fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
    ) -> layout::Node {
        self.overlay.layout(renderer, bounds)
    }

    /// Processes a runtime [`Event`].
    pub fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.overlay
            .on_event(event, layout, cursor, renderer, clipboard, shell)
    }

    /// Returns the current [`mouse::Interaction`] of the [`Element`].
    pub fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.overlay
            .mouse_interaction(layout, cursor, viewport, renderer)
    }

    /// Draws the [`Element`] and its children using the given [`Layout`].
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.overlay.draw(renderer, theme, style, layout, cursor);
    }

    /// Applies a [`widget::Operation`] to the [`Element`].
    pub fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.overlay.operate(layout, renderer, operation);
    }

    /// Returns true if the cursor is over the [`Element`].
    pub fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        self.overlay.is_over(layout, renderer, cursor_position)
    }

    /// Returns the nested overlay of the [`Element`], if there is any.
    pub fn overlay<'b>(
        &'b mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<Element<'b, Message, Theme, Renderer>> {
        self.overlay.overlay(layout, renderer)
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

impl<'a, A, B, Theme, Renderer> Overlay<B, Theme, Renderer>
    for Map<'a, A, B, Theme, Renderer>
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
        operation: &mut dyn widget::Operation<()>,
    ) {
        self.content.operate(layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let event_status = self.content.on_event(
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
        );

        shell.merge(local_shell, self.mapper);

        event_status
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .mouse_interaction(layout, cursor, viewport, renderer)
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

    fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        self.content.is_over(layout, renderer, cursor_position)
    }

    fn overlay<'b>(
        &'b mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<Element<'b, B, Theme, Renderer>> {
        self.content
            .overlay(layout, renderer)
            .map(|overlay| overlay.map(self.mapper))
    }
}
