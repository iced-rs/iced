use iced_core::{Point, Rectangle, Size};

use crate::event;
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::{Clipboard, Event, Layout, Overlay, Shell};

/// An [`Overlay`] container that displays multiple overlay [`overlay::Element`]
/// children.
#[allow(missing_debug_implementations)]
pub struct Group<'a, Message, Renderer> {
    children: Vec<overlay::Element<'a, Message, Renderer>>,
}

impl<'a, Message, Renderer> Group<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'a,
{
    /// Creates an empty [`Group`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`Group`] with the given elements.
    pub fn with_children(
        children: Vec<overlay::Element<'a, Message, Renderer>>,
    ) -> Self {
        Group { children }
    }

    /// Adds an [`overlay::Element`] to the [`Group`].
    pub fn push(
        mut self,
        child: impl Into<overlay::Element<'a, Message, Renderer>>,
    ) -> Self {
        self.children.push(child.into());
        self
    }

    /// Turns the [`Group`] into an overlay [`overlay::Element`].
    pub fn overlay(self) -> overlay::Element<'a, Message, Renderer> {
        overlay::Element::new(Point::ORIGIN, Box::new(self))
    }
}

impl<'a, Message, Renderer> Default for Group<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'a,
{
    fn default() -> Self {
        Self::with_children(Vec::new())
    }
}

impl<'a, Message, Renderer> Overlay<Message, Renderer>
    for Group<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        let translation = position - Point::ORIGIN;

        layout::Node::with_children(
            bounds,
            self.children
                .iter()
                .map(|child| child.layout(renderer, bounds, translation))
                .collect(),
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.children
            .iter_mut()
            .zip(layout.children())
            .map(|(child, layout)| {
                child.on_event(
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &<Renderer as crate::Renderer>::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
    ) {
        for (child, layout) in self.children.iter().zip(layout.children()) {
            child.draw(renderer, theme, style, layout, cursor_position);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(layout.children())
            .map(|(child, layout)| {
                child.mouse_interaction(
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        operation.container(None, &mut |operation| {
            self.children.iter_mut().zip(layout.children()).for_each(
                |(child, layout)| {
                    child.operate(layout, renderer, operation);
                },
            )
        });
    }

    fn is_over(&self, layout: Layout<'_>, cursor_position: Point) -> bool {
        self.children
            .iter()
            .zip(layout.children())
            .any(|(child, layout)| child.is_over(layout, cursor_position))
    }
}

impl<'a, Message, Renderer> From<Group<'a, Message, Renderer>>
    for overlay::Element<'a, Message, Renderer>
where
    Renderer: 'a + crate::Renderer,
    Message: 'a,
{
    fn from(group: Group<'a, Message, Renderer>) -> Self {
        group.overlay()
    }
}
