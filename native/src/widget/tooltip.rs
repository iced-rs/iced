//! Display a widget over another.
use crate::event;
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text;
use crate::widget;
use crate::widget::container;
use crate::widget::overlay;
use crate::widget::{Text, Tree};
use crate::{
    Clipboard, Element, Event, Layout, Length, Padding, Point, Rectangle,
    Shell, Size, Vector, Widget,
};

use std::borrow::Cow;

/// An element to display a widget over another.
#[allow(missing_debug_implementations)]
pub struct Tooltip<'a, Message, Renderer: text::Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: container::StyleSheet + widget::text::StyleSheet,
{
    content: Element<'a, Message, Renderer>,
    tooltip: Text<'a, Renderer>,
    position: Position,
    gap: u16,
    padding: u16,
    snap_within_viewport: bool,
    style: <Renderer::Theme as container::StyleSheet>::Style,
}

impl<'a, Message, Renderer> Tooltip<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: container::StyleSheet + widget::text::StyleSheet,
{
    /// The default padding of a [`Tooltip`] drawn by this renderer.
    const DEFAULT_PADDING: u16 = 5;

    /// Creates a new [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new(
        content: impl Into<Element<'a, Message, Renderer>>,
        tooltip: impl Into<Cow<'a, str>>,
        position: Position,
    ) -> Self {
        Tooltip {
            content: content.into(),
            tooltip: Text::new(tooltip),
            position,
            gap: 0,
            padding: Self::DEFAULT_PADDING,
            snap_within_viewport: true,
            style: Default::default(),
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

    /// Sets whether the [`Tooltip`] is snapped within the viewport.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }

    /// Sets the style of the [`Tooltip`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as container::StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Tooltip<'a, Message, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: container::StyleSheet + widget::text::StyleSheet,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.content.as_widget().width()
    }

    fn height(&self) -> Length {
        self.content.as_widget().height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content.as_widget().layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
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
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor_position,
            viewport,
        );

        let tooltip = &self.tooltip;

        draw(
            renderer,
            theme,
            inherited_style,
            layout,
            cursor_position,
            viewport,
            self.position,
            self.gap,
            self.padding,
            self.snap_within_viewport,
            &self.style,
            |renderer, limits| {
                Widget::<(), Renderer>::layout(tooltip, renderer, limits)
            },
            |renderer, defaults, layout, cursor_position, viewport| {
                Widget::<(), Renderer>::draw(
                    tooltip,
                    &Tree::empty(),
                    renderer,
                    theme,
                    defaults,
                    layout,
                    cursor_position,
                    viewport,
                );
            },
        );
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content.as_widget().overlay(
            &mut tree.children[0],
            layout,
            renderer,
        )
    }
}

impl<'a, Message, Renderer> From<Tooltip<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + text::Renderer,
    Renderer::Theme: container::StyleSheet + widget::text::StyleSheet,
{
    fn from(
        tooltip: Tooltip<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(tooltip)
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

/// Draws a [`Tooltip`].
pub fn draw<Renderer>(
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    inherited_style: &renderer::Style,
    layout: Layout<'_>,
    cursor_position: Point,
    viewport: &Rectangle,
    position: Position,
    gap: u16,
    padding: u16,
    snap_within_viewport: bool,
    style: &<Renderer::Theme as container::StyleSheet>::Style,
    layout_text: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
    draw_text: impl FnOnce(
        &mut Renderer,
        &renderer::Style,
        Layout<'_>,
        Point,
        &Rectangle,
    ),
) where
    Renderer: crate::Renderer,
    Renderer::Theme: container::StyleSheet,
{
    use container::StyleSheet;

    let bounds = layout.bounds();

    if bounds.contains(cursor_position) {
        let gap = f32::from(gap);
        let style = theme.appearance(style);

        let defaults = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        let text_layout = layout_text(
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                snap_within_viewport
                    .then(|| viewport.size())
                    .unwrap_or(Size::INFINITY),
            )
            .pad(Padding::new(padding)),
        );

        let padding = f32::from(padding);
        let text_bounds = text_layout.bounds();
        let x_center = bounds.x + (bounds.width - text_bounds.width) / 2.0;
        let y_center = bounds.y + (bounds.height - text_bounds.height) / 2.0;

        let mut tooltip_bounds = {
            let offset = match position {
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

        if snap_within_viewport {
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
        }

        renderer.with_layer(Rectangle::with_size(Size::INFINITY), |renderer| {
            container::draw_background(renderer, &style, tooltip_bounds);

            draw_text(
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
            )
        });
    }
}
