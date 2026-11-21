//! Override the layout [`Direction`] of a subtree of widgets.
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Operation, Widget};
use crate::core::{Direction, Event, Layout, Length, Rectangle, Shell, Size, Vector};

/// A widget that lays out its contents using a given [`Direction`].
///
/// This is useful to embed a subtree with a reading direction different
/// from the rest of the application.
pub struct Directional<W> {
    content: W,
    direction: Direction,
}

impl<W> Directional<W> {
    /// Creates a [`Directional`] widget with the given [`Direction`] and content.
    pub fn new(direction: Direction, content: W) -> Self {
        Self { content, direction }
    }

    /// Sets the [`Direction`] of the [`Directional`] widget.
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
}

impl<W: widget::Meta> widget::Meta for Directional<W> {
    fn is_void(&self) -> bool {
        self.content.is_void()
    }
}

impl<W, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Directional<W>
where
    W: Widget<Message, Theme, Renderer>,
{
    fn size(&self) -> Size<Length> {
        self.content.size()
    }

    fn tag(&self) -> tree::Tag {
        self.content.tag()
    }

    fn state(&self) -> tree::State {
        self.content.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.content.diff(tree);
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
        _direction: Direction,
    ) {
        self.content.layout(tree, renderer, limits, self.direction);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .operate(tree, layout, viewport, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
        _direction: Direction,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.overlay(
            tree,
            layout,
            renderer,
            viewport,
            translation,
            window,
            self.direction,
        )
    }
}
