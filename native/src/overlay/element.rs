pub use crate::Overlay;

use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::widget;
use crate::{Clipboard, Layout, Point, Rectangle, Shell, Size, Vector};

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

    /// Returns the position of the [`Element`].
    pub fn position(&self) -> Point {
        self.position
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
        B: 'a,
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
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.overlay.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    /// Returns the current [`mouse::Interaction`] of the [`Element`].
    pub fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.overlay.mouse_interaction(
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    /// Draws the [`Element`] and its children using the given [`Layout`].
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        self.overlay
            .draw(renderer, theme, style, layout, cursor_position)
    }

    /// Applies a [`widget::Operation`] to the [`Element`].
    pub fn operate(
        &self,
        layout: Layout<'_>,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.overlay.operate(layout, operation);
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

    fn operate(
        &self,
        layout: Layout<'_>,
        operation: &mut dyn widget::Operation<B>,
    ) {
        struct MapOperation<'a, B> {
            operation: &'a mut dyn widget::Operation<B>,
        }

        impl<'a, T, B> widget::Operation<T> for MapOperation<'a, B> {
            fn container(
                &mut self,
                id: Option<&widget::Id>,
                operate_on_children: &mut dyn FnMut(
                    &mut dyn widget::Operation<T>,
                ),
            ) {
                self.operation.container(id, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }

            fn focusable(
                &mut self,
                state: &mut dyn widget::operation::Focusable,
                id: Option<&widget::Id>,
            ) {
                self.operation.focusable(state, id);
            }

            fn scrollable(
                &mut self,
                state: &mut dyn widget::operation::Scrollable,
                id: Option<&widget::Id>,
            ) {
                self.operation.scrollable(state, id);
            }

            fn text_input(
                &mut self,
                state: &mut dyn widget::operation::TextInput,
                id: Option<&widget::Id>,
            ) {
                self.operation.text_input(state, id)
            }
        }

        self.content
            .operate(layout, &mut MapOperation { operation });
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let event_status = self.content.on_event(
            event,
            layout,
            cursor_position,
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
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.mouse_interaction(
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        self.content
            .draw(renderer, theme, style, layout, cursor_position)
    }
}
