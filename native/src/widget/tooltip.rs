//! Display a widget over another.
use std::hash::Hash;

use iced_core::Rectangle;

use crate::{
    event, layout, overlay, Clipboard, Element, Event, Hasher, Layout, Length,
    Point, Size, Vector, Widget,
};

/// An element to display a widget over another.
#[allow(missing_debug_implementations)]
pub struct Tooltip<'a, Message, Renderer: self::Renderer> {
    content: Element<'a, Message, Renderer>,
    tooltip: Element<'a, Message, Renderer>,
    tooltip_position: TooltipPosition,
}

impl<'a, Message, Renderer> Tooltip<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// Creates an empty [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new<T, H>(
        content: T,
        tooltip: H,
        tooltip_position: TooltipPosition,
    ) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
        H: Into<Element<'a, Message, Renderer>>,
    {
        Tooltip {
            content: content.into(),
            tooltip: tooltip.into(),
            tooltip_position,
        }
    }
}

/// The position of the tooltip. Defaults to following the cursor.
#[derive(Debug, PartialEq)]
pub enum TooltipPosition {
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

impl Default for TooltipPosition {
    fn default() -> Self {
        TooltipPosition::FollowCursor
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Tooltip<'a, Message, Renderer>
where
    Renderer: self::Renderer,
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
        renderer.draw(
            defaults,
            cursor_position,
            &self.content,
            layout,
            viewport,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash_layout(state);
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        overlay_content_bounds: Option<Rectangle>,
        cursor_position: Point,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let bounds = layout.bounds();

        if bounds.contains(cursor_position) {
            let mut position = cursor_position;

            if let Some(content_bounds) = overlay_content_bounds {
                if TooltipPosition::FollowCursor != self.tooltip_position {
                    match self.tooltip_position {
                        TooltipPosition::Top | TooltipPosition::Bottom => {
                            let x = bounds.x + bounds.width * 0.5
                                - content_bounds.width * 0.5;

                            position = match self.tooltip_position {
                                TooltipPosition::Top => Point::new(
                                    x,
                                    bounds.y - content_bounds.height,
                                ),
                                TooltipPosition::Bottom => Point::new(
                                    x,
                                    bounds.y
                                        + bounds.height
                                        + content_bounds.height,
                                ),
                                _ => unreachable!(),
                            };
                        }
                        TooltipPosition::Left | TooltipPosition::Right => {
                            let y =
                                bounds.center_y() + content_bounds.height * 0.5;

                            position = match self.tooltip_position {
                                TooltipPosition::Left => Point::new(
                                    bounds.x - content_bounds.width,
                                    y,
                                ),
                                TooltipPosition::Right => {
                                    Point::new(bounds.x + bounds.width, y)
                                }
                                _ => unreachable!(),
                            };
                        }
                        _ => {}
                    }
                }
            }

            Some(overlay::Element::new(
                position,
                Box::new(Overlay::new(&self.tooltip)),
            ))
        } else {
            None
        }
    }
}

struct Overlay<'a, Message, Renderer: self::Renderer> {
    content: &'a Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer: self::Renderer> Overlay<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a,
{
    pub fn new(content: &'a Element<'a, Message, Renderer>) -> Self {
        Self { content }
    }
}

impl<'a, Message, Renderer> crate::Overlay<Message, Renderer>
    for Overlay<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn layout(
        &self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
    ) -> layout::Node {
        let space_below = bounds.height - position.y;
        let space_above = position.y;

        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                bounds.width - position.x,
                if space_below > space_above {
                    space_below
                } else {
                    space_above
                },
            ),
        )
        .width(self.content.width());

        let mut node = self.content.layout(renderer, &limits);

        node.move_to(position - Vector::new(0.0, node.size().height));

        node
    }

    fn hash_layout(&self, state: &mut Hasher, position: Point) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        (position.x as u32).hash(state);
        (position.y as u32).hash(state);
        self.content.hash_layout(state);
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            cursor_position,
            &self.content,
            layout,
            viewport,
        )
    }
}

/// The renderer of a [`Tooltip`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Tooltip`] in your user interface.
///
/// [`Tooltip`]: struct.Tooltip.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        cursor_position: Point,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Tooltip<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        column: Tooltip<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
