//! Display a widget over another.
use crate::container;
use crate::core::event::{self, Event};
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::widget::{self, Widget};
use crate::core::{
    Clipboard, Element, Length, Padding, Pixels, Point, Rectangle, Shell, Size,
    Vector,
};
use crate::Text;

/// An element to display a widget over another.
#[allow(missing_debug_implementations)]
pub struct Tooltip<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: container::StyleSheet + crate::text::StyleSheet,
    Renderer: text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    tooltip: Text<'a, Theme, Renderer>,
    position: Position,
    gap: f32,
    padding: f32,
    snap_within_viewport: bool,
    style: <Theme as container::StyleSheet>::Style,
}

impl<'a, Message, Theme, Renderer> Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet + crate::text::StyleSheet,
    Renderer: text::Renderer,
{
    /// The default padding of a [`Tooltip`] drawn by this renderer.
    const DEFAULT_PADDING: f32 = 5.0;

    /// Creates a new [`Tooltip`].
    ///
    /// [`Tooltip`]: struct.Tooltip.html
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        tooltip: impl Into<Text<'a, Theme, Renderer>>,
        position: Position,
    ) -> Self {
        Tooltip {
            content: content.into(),
            tooltip: tooltip.into(),
            position,
            gap: 0.0,
            padding: Self::DEFAULT_PADDING,
            snap_within_viewport: true,
            style: Default::default(),
        }
    }

    /// Sets the gap between the content and its [`Tooltip`].
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets the padding of the [`Tooltip`].
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
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
        style: impl Into<<Theme as container::StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Tooltip<'a, Message, Theme, Renderer>
where
    Theme: container::StyleSheet + crate::text::StyleSheet,
    Renderer: text::Renderer,
{
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.content),
            widget::Tree::new(&self.tooltip as &dyn Widget<Message, _, _>),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.content.as_widget(), &self.tooltip]);
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State::default())
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        let state = tree.state.downcast_mut::<State>();

        let was_idle = *state == State::Idle;

        *state = cursor
            .position_over(layout.bounds())
            .map(|cursor_position| State::Hovered { cursor_position })
            .unwrap_or_default();

        let is_idle = *state == State::Idle;

        if was_idle != is_idle {
            shell.invalidate_layout();
        }

        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        let mut children = tree.children.iter_mut();

        let content = self.content.as_widget_mut().overlay(
            children.next().unwrap(),
            layout,
            renderer,
        );

        let tooltip = if let State::Hovered { cursor_position } = *state {
            Some(overlay::Element::new(
                layout.position(),
                Box::new(Overlay {
                    tooltip: &self.tooltip,
                    state: children.next().unwrap(),
                    cursor_position,
                    content_bounds: layout.bounds(),
                    snap_within_viewport: self.snap_within_viewport,
                    position: self.position,
                    gap: self.gap,
                    padding: self.padding,
                    style: &self.style,
                }),
            ))
        } else {
            None
        };

        if content.is_some() || tooltip.is_some() {
            Some(
                overlay::Group::with_children(
                    content.into_iter().chain(tooltip).collect(),
                )
                .overlay(),
            )
        } else {
            None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Tooltip<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: container::StyleSheet + crate::text::StyleSheet + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        tooltip: Tooltip<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum State {
    #[default]
    Idle,
    Hovered {
        cursor_position: Point,
    },
}

struct Overlay<'a, 'b, Theme, Renderer>
where
    Theme: container::StyleSheet + widget::text::StyleSheet,
    Renderer: text::Renderer,
{
    tooltip: &'b Text<'a, Theme, Renderer>,
    state: &'b mut widget::Tree,
    cursor_position: Point,
    content_bounds: Rectangle,
    snap_within_viewport: bool,
    position: Position,
    gap: f32,
    padding: f32,
    style: &'b <Theme as container::StyleSheet>::Style,
}

impl<'a, 'b, Message, Theme, Renderer>
    overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, Theme, Renderer>
where
    Theme: container::StyleSheet + widget::text::StyleSheet,
    Renderer: text::Renderer,
{
    fn layout(
        &mut self,
        renderer: &Renderer,
        bounds: Size,
        position: Point,
        _translation: Vector,
    ) -> layout::Node {
        let viewport = Rectangle::with_size(bounds);

        let text_layout = Widget::<(), Theme, Renderer>::layout(
            self.tooltip,
            self.state,
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                self.snap_within_viewport
                    .then(|| viewport.size())
                    .unwrap_or(Size::INFINITY),
            )
            .shrink(Padding::new(self.padding)),
        );

        let text_bounds = text_layout.bounds();
        let x_center =
            position.x + (self.content_bounds.width - text_bounds.width) / 2.0;
        let y_center = position.y
            + (self.content_bounds.height - text_bounds.height) / 2.0;

        let mut tooltip_bounds = {
            let offset = match self.position {
                Position::Top => Vector::new(
                    x_center,
                    position.y - text_bounds.height - self.gap - self.padding,
                ),
                Position::Bottom => Vector::new(
                    x_center,
                    position.y
                        + self.content_bounds.height
                        + self.gap
                        + self.padding,
                ),
                Position::Left => Vector::new(
                    position.x - text_bounds.width - self.gap - self.padding,
                    y_center,
                ),
                Position::Right => Vector::new(
                    position.x
                        + self.content_bounds.width
                        + self.gap
                        + self.padding,
                    y_center,
                ),
                Position::FollowCursor => {
                    let translation = position - self.content_bounds.position();

                    Vector::new(
                        self.cursor_position.x,
                        self.cursor_position.y - text_bounds.height,
                    ) + translation
                }
            };

            Rectangle {
                x: offset.x - self.padding,
                y: offset.y - self.padding,
                width: text_bounds.width + self.padding * 2.0,
                height: text_bounds.height + self.padding * 2.0,
            }
        };

        if self.snap_within_viewport {
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

        layout::Node::with_children(
            tooltip_bounds.size(),
            vec![text_layout.translate(Vector::new(self.padding, self.padding))],
        )
        .translate(Vector::new(tooltip_bounds.x, tooltip_bounds.y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        let style = container::StyleSheet::appearance(theme, self.style);

        container::draw_background(renderer, &style, layout.bounds());

        let defaults = renderer::Style {
            text_color: style.text_color.unwrap_or(inherited_style.text_color),
        };

        Widget::<(), Theme, Renderer>::draw(
            self.tooltip,
            self.state,
            renderer,
            theme,
            &defaults,
            layout.children().next().unwrap(),
            cursor_position,
            &Rectangle::with_size(Size::INFINITY),
        );
    }

    fn is_over(
        &self,
        _layout: Layout<'_>,
        _renderer: &Renderer,
        _cursor_position: Point,
    ) -> bool {
        false
    }
}
