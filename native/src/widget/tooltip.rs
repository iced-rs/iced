//! Display a widget over another.
use std::hash::Hash;

use iced_core::Rectangle;

use crate::widget::text::{self, Text};
use crate::{
    event, layout, Clipboard, Element, Event, Hasher, Layout, Length, Point,
    Widget,
};

/// An element to display a widget over another.
#[allow(missing_debug_implementations)]
pub struct Tooltip<'a, Message, Renderer: self::Renderer + text::Renderer> {
    content: Element<'a, Message, Renderer>,
    tooltip: Text<Renderer>,
    position: Position,
}

impl<'a, Message, Renderer> Tooltip<'a, Message, Renderer>
where
    Renderer: self::Renderer + text::Renderer,
{
    /// Creates an empty [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        tooltip: Text<Renderer>,
        position: Position,
    ) -> Self {
        Tooltip {
            content: content.into(),
            tooltip,
            position,
        }
    }
}

/// The position of the tooltip. Defaults to following the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    /// The tooltip will follow the cursor.
    FollowCursor,
    /// The tooltip will appear on the top of the widget.
    Top,
    /// The tooltip will appear on the bottom of the widget.
    Bottom,
    /// The tooltip will appear on the left of the widget.
    Left,
    /// The tooltip will appear on the right of the widget.
    Right,
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Tooltip<'a, Message, Renderer>
where
    Renderer: self::Renderer + text::Renderer,
{
    fn width(&self) -> Length {
        self.content.width()
    }

    fn height(&self) -> Length {
        self.content.height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        self.content.widget.on_event(
            event,
            layout,
            cursor_position,
            messages,
            renderer,
            clipboard,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        self::Renderer::draw(
            renderer,
            defaults,
            cursor_position,
            layout,
            viewport,
            &self.content,
            &self.tooltip,
            self.position,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash_layout(state);
    }
}

/// The renderer of a [`Tooltip`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Tooltip`] in your user interface.
///
/// [`Tooltip`]: struct.Tooltip.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + text::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        cursor_position: Point,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
        content: &Element<'_, Message, Self>,
        tooltip: &Text<Self>,
        position: Position,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Tooltip<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + text::Renderer,
    Message: 'a,
{
    fn from(
        column: Tooltip<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
