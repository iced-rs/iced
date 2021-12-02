//! Display a widget over another.
use std::hash::Hash;

use iced_core::Rectangle;

use crate::event;
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text;
use crate::widget::container;
use crate::widget::text::Text;
use crate::{
    Clipboard, Element, Event, Hasher, Layout, Length, Padding, Point, Shell,
    Size, Vector, Widget,
};

/// An element to display a widget over another.
#[allow(missing_debug_implementations)]
pub struct Tooltip<'a, Message, Renderer: text::Renderer> {
    content: Element<'a, Message, Renderer>,
    tooltip: Text<Renderer>,
    position: Position,
    style_sheet: Box<dyn container::StyleSheet + 'a>,
    gap: u16,
    padding: u16,
}

impl<'a, Message, Renderer> Tooltip<'a, Message, Renderer>
where
    Renderer: text::Renderer,
{
    /// The default padding of a [`Tooltip`] drawn by this renderer.
    const DEFAULT_PADDING: u16 = 5;

    /// Creates an empty [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        tooltip: impl ToString,
        position: Position,
    ) -> Self {
        Tooltip {
            content: content.into(),
            tooltip: Text::new(tooltip.to_string()),
            position,
            style_sheet: Default::default(),
            gap: 0,
            padding: Self::DEFAULT_PADDING,
        }
    }

    /// Sets the size of the text of the [`Tooltip`].
    pub fn size(mut self, size: u16) -> Self {
        self.tooltip = self.tooltip.size(size);
        self
    }

    /// Sets the font of the [`Tooltip`].
    ///
    /// [`Font`]: Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.tooltip = self.tooltip.font(font);
        self
    }

    /// Sets the gap between the content and its [`Tooltip`].
    pub fn gap(mut self, gap: u16) -> Self {
        self.gap = gap;
        self
    }

    /// Sets the padding of the [`Tooltip`].
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the style of the [`Tooltip`].
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn container::StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
        self
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
    Renderer: text::Renderer,
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.widget.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> mouse::Interaction {
        self.content
            .mouse_interaction(layout, cursor_position, viewport)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.draw(
            renderer,
            inherited_style,
            layout,
            cursor_position,
            viewport,
        );

        let bounds = layout.bounds();

        if bounds.contains(cursor_position) {
            let gap = f32::from(self.gap);
            let style = self.style_sheet.style();

            let defaults = renderer::Style {
                text_color: style
                    .text_color
                    .unwrap_or(inherited_style.text_color),
            };

            let text_layout = Widget::<(), Renderer>::layout(
                &self.tooltip,
                renderer,
                &layout::Limits::new(Size::ZERO, viewport.size())
                    .pad(Padding::new(self.padding)),
            );

            let padding = f32::from(self.padding);
            let text_bounds = text_layout.bounds();
            let x_center = bounds.x + (bounds.width - text_bounds.width) / 2.0;
            let y_center =
                bounds.y + (bounds.height - text_bounds.height) / 2.0;

            let mut tooltip_bounds = {
                let offset = match self.position {
                    Position::Top => Vector::new(
                        x_center,
                        bounds.y - text_bounds.height - gap - padding,
                    ),
                    Position::Bottom => Vector::new(
                        x_center,
                        bounds.y + bounds.height + gap + padding,
                    ),
                    Position::Left => Vector::new(
                        bounds.x - text_bounds.width - gap - padding,
                        y_center,
                    ),
                    Position::Right => Vector::new(
                        bounds.x + bounds.width + gap + padding,
                        y_center,
                    ),
                    Position::FollowCursor => Vector::new(
                        cursor_position.x,
                        cursor_position.y - text_bounds.height,
                    ),
                };

                Rectangle {
                    x: offset.x - padding,
                    y: offset.y - padding,
                    width: text_bounds.width + padding * 2.0,
                    height: text_bounds.height + padding * 2.0,
                }
            };

            if tooltip_bounds.x < viewport.x {
                tooltip_bounds.x = viewport.x;
            } else if viewport.x + viewport.width
                < tooltip_bounds.x + tooltip_bounds.width
            {
                tooltip_bounds.x =
                    viewport.x + viewport.width - tooltip_bounds.width;
            }

            if tooltip_bounds.y < viewport.y {
                tooltip_bounds.y = viewport.y;
            } else if viewport.y + viewport.height
                < tooltip_bounds.y + tooltip_bounds.height
            {
                tooltip_bounds.y =
                    viewport.y + viewport.height - tooltip_bounds.height;
            }

            renderer.with_layer(*viewport, |renderer| {
                container::draw_background(renderer, &style, tooltip_bounds);

                Widget::<(), Renderer>::draw(
                    &self.tooltip,
                    renderer,
                    &defaults,
                    Layout::with_offset(
                        Vector::new(
                            tooltip_bounds.x + padding,
                            tooltip_bounds.y + padding,
                        ),
                        &text_layout,
                    ),
                    cursor_position,
                    viewport,
                );
            });
        }
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash_layout(state);
    }
}

impl<'a, Message, Renderer> From<Tooltip<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + text::Renderer,
    Message: 'a,
{
    fn from(
        column: Tooltip<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
