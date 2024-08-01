use crate::event;
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::{Clipboard, Event, Layout, Overlay, Point, Rectangle, Shell, Size};

/// An [`Overlay`] container that displays multiple overlay [`overlay::Element`]
/// children.
#[allow(missing_debug_implementations)]
pub struct Group<'a, Message, Theme, Renderer> {
    children: Vec<overlay::Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Group<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::Renderer,
{
    /// Creates an empty [`Group`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a [`Group`] with the given elements.
    pub fn with_children(
        children: Vec<overlay::Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Group { children }
    }

    /// Adds an [`overlay::Element`] to the [`Group`].
    pub fn push(
        mut self,
        child: impl Into<overlay::Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.children.push(child.into());
        self
    }

    /// Turns the [`Group`] into an overlay [`overlay::Element`].
    pub fn overlay(self) -> overlay::Element<'a, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }
}

impl<'a, Message, Theme, Renderer> Default
    for Group<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::Renderer,
{
    fn default() -> Self {
        Self::with_children(Vec::new())
    }
}

impl<'a, Message, Theme, Renderer> Overlay<Message, Theme, Renderer>
    for Group<'a, Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        layout::Node::with_children(
            bounds,
            self.children
                .iter_mut()
                .map(|child| child.layout(renderer, bounds))
                .collect(),
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
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
                    cursor,
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
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        for (child, layout) in self.children.iter().zip(layout.children()) {
            child.draw(renderer, theme, style, layout, cursor);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(layout.children())
            .map(|(child, layout)| {
                child.mouse_interaction(layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.children.iter_mut().zip(layout.children()).for_each(
                |(child, layout)| {
                    child.operate(layout, renderer, operation);
                },
            );
        });
    }

    fn is_over(
        &self,
        layout: Layout<'_>,
        renderer: &Renderer,
        cursor_position: Point,
    ) -> bool {
        self.children
            .iter()
            .zip(layout.children())
            .any(|(child, layout)| {
                child.is_over(layout, renderer, cursor_position)
            })
    }

    fn overlay<'b>(
        &'b mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let children = self
            .children
            .iter_mut()
            .zip(layout.children())
            .filter_map(|(child, layout)| child.overlay(layout, renderer))
            .collect::<Vec<_>>();

        (!children.is_empty()).then(|| Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Group<'a, Message, Theme, Renderer>>
    for overlay::Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::Renderer,
{
    fn from(group: Group<'a, Message, Theme, Renderer>) -> Self {
        group.overlay()
    }
}
