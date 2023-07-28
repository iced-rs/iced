//! Navigate an endless amount of content with a scrollbar.
use crate::core::event::{self, Event};
use crate::core::keyboard;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::operation::{self, Operation};
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Background, Clipboard, Color, Element, Layout, Length, Pixels, Point,
    Rectangle, Shell, Size, Vector, Widget,
};
use crate::runtime::Command;

pub use crate::style::scrollable::{Scrollbar, Scroller, StyleSheet};
pub use operation::scrollable::{AbsoluteOffset, RelativeOffset};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message, Renderer = crate::Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    width: Length,
    height: Length,
    direction: Direction,
    content: Element<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> Message + 'a>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Scrollable<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Scrollable`].
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Scrollable {
            id: None,
            width: Length::Shrink,
            height: Length::Shrink,
            direction: Default::default(),
            content: content.into(),
            on_scroll: None,
            style: Default::default(),
        }
    }

    /// Sets the [`Id`] of the [`Scrollable`].
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Direction`] of the [`Scrollable`] .
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets a function to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the [`Viewport`] of the [`Scrollable`]
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Sets the style of the [`Scrollable`] .
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }
}

/// The direction of [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Vertical scrolling
    Vertical(Properties),
    /// Horizontal scrolling
    Horizontal(Properties),
    /// Both vertical and horizontal scrolling
    Both {
        /// The properties of the vertical scrollbar.
        vertical: Properties,
        /// The properties of the horizontal scrollbar.
        horizontal: Properties,
    },
}

impl Direction {
    /// Returns the [`Properties`] of the horizontal scrollbar, if any.
    pub fn horizontal(&self) -> Option<&Properties> {
        match self {
            Self::Horizontal(properties) => Some(properties),
            Self::Both { horizontal, .. } => Some(horizontal),
            _ => None,
        }
    }

    /// Returns the [`Properties`] of the vertical scrollbar, if any.
    pub fn vertical(&self) -> Option<&Properties> {
        match self {
            Self::Vertical(properties) => Some(properties),
            Self::Both { vertical, .. } => Some(vertical),
            _ => None,
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::Vertical(Properties::default())
    }
}

/// Properties of a scrollbar within a [`Scrollable`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Properties {
    width: f32,
    margin: f32,
    scroller_width: f32,
    alignment: Alignment,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            width: 10.0,
            margin: 0.0,
            scroller_width: 10.0,
            alignment: Alignment::Start,
        }
    }
}

impl Properties {
    /// Creates new [`Properties`] for use in a [`Scrollable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the scrollbar width of the [`Scrollable`] .
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = width.into().0.max(0.0);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollable`] .
    pub fn margin(mut self, margin: impl Into<Pixels>) -> Self {
        self.margin = margin.into().0;
        self
    }

    /// Sets the scroller width of the [`Scrollable`] .
    pub fn scroller_width(mut self, scroller_width: impl Into<Pixels>) -> Self {
        self.scroller_width = scroller_width.into().0.max(0.0);
        self
    }

    /// Sets the alignment of the [`Scrollable`] .
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Alignment of the scrollable's content relative to it's [`Viewport`] in one direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Alignment {
    /// Content is aligned to the start of the [`Viewport`].
    #[default]
    Start,
    /// Content is aligned to the end of the [`Viewport`]
    End,
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.height,
            &self.direction,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let translation =
            state.translation(self.direction, bounds, content_bounds);

        operation.scrollable(
            state,
            self.id.as_ref().map(|id| &id.0),
            bounds,
            translation,
        );

        operation.container(
            self.id.as_ref().map(|id| &id.0),
            bounds,
            &mut |operation| {
                self.content.as_widget().operate(
                    &mut tree.children[0],
                    layout.children().next().unwrap(),
                    renderer,
                    operation,
                );
            },
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        update(
            tree.state.downcast_mut::<State>(),
            event,
            layout,
            cursor,
            clipboard,
            shell,
            self.direction,
            &self.on_scroll,
            |event, layout, cursor, clipboard, shell, viewport| {
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
            },
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        draw(
            tree.state.downcast_ref::<State>(),
            renderer,
            theme,
            layout,
            cursor,
            self.direction,
            &self.style,
            |renderer, layout, cursor, viewport| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor,
                    viewport,
                )
            },
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(
            tree.state.downcast_ref::<State>(),
            layout,
            cursor,
            self.direction,
            |layout, cursor, viewport| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor,
                    viewport,
                    renderer,
                )
            },
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
            )
            .map(|overlay| {
                let bounds = layout.bounds();
                let content_layout = layout.children().next().unwrap();
                let content_bounds = content_layout.bounds();
                let translation = tree
                    .state
                    .downcast_ref::<State>()
                    .translation(self.direction, bounds, content_bounds);

                overlay.translate(Vector::new(-translation.x, -translation.y))
            })
    }
}

impl<'a, Message, Renderer> From<Scrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(
        text_input: Scrollable<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(text_input)
    }
}

/// The identifier of a [`Scrollable`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}

/// Produces a [`Command`] that snaps the [`Scrollable`] with the given [`Id`]
/// to the provided `percentage` along the x & y axis.
pub fn snap_to<Message: 'static>(
    id: Id,
    offset: RelativeOffset,
) -> Command<Message> {
    Command::widget(operation::scrollable::snap_to(id.0, offset))
}

/// Produces a [`Command`] that scrolls the [`Scrollable`] with the given [`Id`]
/// to the provided [`AbsoluteOffset`] along the x & y axis.
pub fn scroll_to<Message: 'static>(
    id: Id,
    offset: AbsoluteOffset,
) -> Command<Message> {
    Command::widget(operation::scrollable::scroll_to(id.0, offset))
}

/// Computes the layout of a [`Scrollable`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    direction: &Direction,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.width(width).height(height);

    let child_limits = layout::Limits::new(
        Size::new(limits.min().width, limits.min().height),
        Size::new(
            if direction.horizontal().is_some() {
                f32::INFINITY
            } else {
                limits.max().width
            },
            if direction.vertical().is_some() {
                f32::MAX
            } else {
                limits.max().height
            },
        ),
    );

    let content = layout_content(renderer, &child_limits);
    let size = limits.resolve(content.size());

    layout::Node::with_children(size, vec![content])
}

/// Processes an [`Event`] and updates the [`State`] of a [`Scrollable`]
/// accordingly.
pub fn update<Message>(
    state: &mut State,
    event: Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    direction: Direction,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Message + '_>>,
    update_content: impl FnOnce(
        Event,
        Layout<'_>,
        mouse::Cursor,
        &mut dyn Clipboard,
        &mut Shell<'_, Message>,
        &Rectangle,
    ) -> event::Status,
) -> event::Status {
    let bounds = layout.bounds();
    let cursor_over_scrollable = cursor.position_over(bounds);

    let content = layout.children().next().unwrap();
    let content_bounds = content.bounds();

    let scrollbars = Scrollbars::new(state, direction, bounds, content_bounds);

    let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) =
        scrollbars.is_mouse_over(cursor);

    let event_status = {
        let cursor = match cursor_over_scrollable {
            Some(cursor_position)
                if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) =>
            {
                mouse::Cursor::Available(
                    cursor_position
                        + state.translation(direction, bounds, content_bounds),
                )
            }
            _ => mouse::Cursor::Unavailable,
        };

        let translation = state.translation(direction, bounds, content_bounds);

        update_content(
            event.clone(),
            content,
            cursor,
            clipboard,
            shell,
            &Rectangle {
                y: bounds.y + translation.y,
                x: bounds.x + translation.x,
                ..bounds
            },
        )
    };

    if let event::Status::Captured = event_status {
        return event::Status::Captured;
    }

    if let Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) = event
    {
        state.keyboard_modifiers = modifiers;

        return event::Status::Ignored;
    }

    match event {
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            if cursor_over_scrollable.is_none() {
                return event::Status::Ignored;
            }

            let delta = match delta {
                mouse::ScrollDelta::Lines { x, y } => {
                    // TODO: Configurable speed/friction (?)
                    let movement = if state.keyboard_modifiers.shift() {
                        Vector::new(y, x)
                    } else {
                        Vector::new(x, y)
                    };

                    movement * 60.0
                }
                mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
            };

            state.scroll(delta, direction, bounds, content_bounds);

            notify_on_scroll(state, on_scroll, bounds, content_bounds, shell);

            return event::Status::Captured;
        }
        Event::Touch(event)
            if state.scroll_area_touched_at.is_some()
                || !mouse_over_y_scrollbar && !mouse_over_x_scrollbar =>
        {
            match event {
                touch::Event::FingerPressed { .. } => {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored
                    };

                    state.scroll_area_touched_at = Some(cursor_position);
                }
                touch::Event::FingerMoved { .. } => {
                    if let Some(scroll_box_touched_at) =
                        state.scroll_area_touched_at
                    {
                        let Some(cursor_position) = cursor.position() else {
                            return event::Status::Ignored
                        };

                        let delta = Vector::new(
                            cursor_position.x - scroll_box_touched_at.x,
                            cursor_position.y - scroll_box_touched_at.y,
                        );

                        state.scroll(delta, direction, bounds, content_bounds);

                        state.scroll_area_touched_at = Some(cursor_position);

                        notify_on_scroll(
                            state,
                            on_scroll,
                            bounds,
                            content_bounds,
                            shell,
                        );
                    }
                }
                touch::Event::FingerLifted { .. }
                | touch::Event::FingerLost { .. } => {
                    state.scroll_area_touched_at = None;
                }
            }

            return event::Status::Captured;
        }
        _ => {}
    }

    if let Some(scroller_grabbed_at) = state.y_scroller_grabbed_at {
        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.y_scroller_grabbed_at = None;

                return event::Status::Captured;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let Some(scrollbar) = scrollbars.y {
                    let Some(cursor_position) = cursor.position() else {
                        return event::Status::Ignored
                    };

                    state.scroll_y_to(
                        scrollbar.scroll_percentage_y(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );

                    return event::Status::Captured;
                }
            }
            _ => {}
        }
    } else if mouse_over_y_scrollbar {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let Some(cursor_position) = cursor.position() else {
                    return event::Status::Ignored
                };

                if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                    (scrollbars.grab_y_scroller(cursor_position), scrollbars.y)
                {
                    state.scroll_y_to(
                        scrollbar.scroll_percentage_y(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    state.y_scroller_grabbed_at = Some(scroller_grabbed_at);

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );
                }

                return event::Status::Captured;
            }
            _ => {}
        }
    }

    if let Some(scroller_grabbed_at) = state.x_scroller_grabbed_at {
        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.x_scroller_grabbed_at = None;

                return event::Status::Captured;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                let Some(cursor_position) = cursor.position() else {
                    return event::Status::Ignored
                };

                if let Some(scrollbar) = scrollbars.x {
                    state.scroll_x_to(
                        scrollbar.scroll_percentage_x(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );
                }

                return event::Status::Captured;
            }
            _ => {}
        }
    } else if mouse_over_x_scrollbar {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let Some(cursor_position) = cursor.position() else {
                    return event::Status::Ignored
                };

                if let (Some(scroller_grabbed_at), Some(scrollbar)) =
                    (scrollbars.grab_x_scroller(cursor_position), scrollbars.x)
                {
                    state.scroll_x_to(
                        scrollbar.scroll_percentage_x(
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    state.x_scroller_grabbed_at = Some(scroller_grabbed_at);

                    notify_on_scroll(
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );

                    return event::Status::Captured;
                }
            }
            _ => {}
        }
    }

    event::Status::Ignored
}

/// Computes the current [`mouse::Interaction`] of a [`Scrollable`].
pub fn mouse_interaction(
    state: &State,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    direction: Direction,
    content_interaction: impl FnOnce(
        Layout<'_>,
        mouse::Cursor,
        &Rectangle,
    ) -> mouse::Interaction,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let cursor_over_scrollable = cursor.position_over(bounds);

    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let scrollbars = Scrollbars::new(state, direction, bounds, content_bounds);

    let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) =
        scrollbars.is_mouse_over(cursor);

    if (mouse_over_x_scrollbar || mouse_over_y_scrollbar)
        || state.scrollers_grabbed()
    {
        mouse::Interaction::Idle
    } else {
        let translation = state.translation(direction, bounds, content_bounds);

        let cursor = match cursor_over_scrollable {
            Some(cursor_position)
                if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) =>
            {
                mouse::Cursor::Available(cursor_position + translation)
            }
            _ => mouse::Cursor::Unavailable,
        };

        content_interaction(
            content_layout,
            cursor,
            &Rectangle {
                y: bounds.y + translation.y,
                x: bounds.x + translation.x,
                ..bounds
            },
        )
    }
}

/// Draws a [`Scrollable`].
pub fn draw<Renderer>(
    state: &State,
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    direction: Direction,
    style: &<Renderer::Theme as StyleSheet>::Style,
    draw_content: impl FnOnce(&mut Renderer, Layout<'_>, mouse::Cursor, &Rectangle),
) where
    Renderer: crate::core::Renderer,
    Renderer::Theme: StyleSheet,
{
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let scrollbars = Scrollbars::new(state, direction, bounds, content_bounds);

    let cursor_over_scrollable = cursor.position_over(bounds);
    let (mouse_over_y_scrollbar, mouse_over_x_scrollbar) =
        scrollbars.is_mouse_over(cursor);

    let translation = state.translation(direction, bounds, content_bounds);

    let cursor = match cursor_over_scrollable {
        Some(cursor_position)
            if !(mouse_over_x_scrollbar || mouse_over_y_scrollbar) =>
        {
            mouse::Cursor::Available(cursor_position + translation)
        }
        _ => mouse::Cursor::Unavailable,
    };

    // Draw inner content
    if scrollbars.active() {
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(
                Vector::new(-translation.x, -translation.y),
                |renderer| {
                    draw_content(
                        renderer,
                        content_layout,
                        cursor,
                        &Rectangle {
                            y: bounds.y + translation.y,
                            x: bounds.x + translation.x,
                            ..bounds
                        },
                    );
                },
            );
        });

        let draw_scrollbar =
            |renderer: &mut Renderer,
             style: Scrollbar,
             scrollbar: &internals::Scrollbar| {
                //track
                if scrollbar.bounds.width > 0.0
                    && scrollbar.bounds.height > 0.0
                    && (style.background.is_some()
                        || (style.border_color != Color::TRANSPARENT
                            && style.border_width > 0.0))
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scrollbar.bounds,
                            border_radius: style.border_radius,
                            border_width: style.border_width,
                            border_color: style.border_color,
                        },
                        style
                            .background
                            .unwrap_or(Background::Color(Color::TRANSPARENT)),
                    );
                }

                //thumb
                if scrollbar.scroller.bounds.width > 0.0
                    && scrollbar.scroller.bounds.height > 0.0
                    && (style.scroller.color != Color::TRANSPARENT
                        || (style.scroller.border_color != Color::TRANSPARENT
                            && style.scroller.border_width > 0.0))
                {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: scrollbar.scroller.bounds,
                            border_radius: style.scroller.border_radius,
                            border_width: style.scroller.border_width,
                            border_color: style.scroller.border_color,
                        },
                        style.scroller.color,
                    );
                }
            };

        renderer.with_layer(
            Rectangle {
                width: bounds.width + 2.0,
                height: bounds.height + 2.0,
                ..bounds
            },
            |renderer| {
                //draw y scrollbar
                if let Some(scrollbar) = scrollbars.y {
                    let style = if state.y_scroller_grabbed_at.is_some() {
                        theme.dragging(style)
                    } else if cursor_over_scrollable.is_some() {
                        theme.hovered(style, mouse_over_y_scrollbar)
                    } else {
                        theme.active(style)
                    };

                    draw_scrollbar(renderer, style, &scrollbar);
                }

                //draw x scrollbar
                if let Some(scrollbar) = scrollbars.x {
                    let style = if state.x_scroller_grabbed_at.is_some() {
                        theme.dragging_horizontal(style)
                    } else if cursor_over_scrollable.is_some() {
                        theme.hovered_horizontal(style, mouse_over_x_scrollbar)
                    } else {
                        theme.active_horizontal(style)
                    };

                    draw_scrollbar(renderer, style, &scrollbar);
                }
            },
        );
    } else {
        draw_content(
            renderer,
            content_layout,
            cursor,
            &Rectangle {
                x: bounds.x + translation.x,
                y: bounds.y + translation.y,
                ..bounds
            },
        );
    }
}

fn notify_on_scroll<Message>(
    state: &mut State,
    on_scroll: &Option<Box<dyn Fn(Viewport) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_scroll) = on_scroll {
        if content_bounds.width <= bounds.width
            && content_bounds.height <= bounds.height
        {
            return;
        }

        let viewport = Viewport {
            offset_x: state.offset_x,
            offset_y: state.offset_y,
            bounds,
            content_bounds,
        };

        // Don't publish redundant viewports to shell
        if let Some(last_notified) = state.last_notified {
            let last_relative_offset = last_notified.relative_offset();
            let current_relative_offset = viewport.relative_offset();

            let last_absolute_offset = last_notified.absolute_offset();
            let current_absolute_offset = viewport.absolute_offset();

            let unchanged = |a: f32, b: f32| {
                (a - b).abs() <= f32::EPSILON || (a.is_nan() && b.is_nan())
            };

            if unchanged(last_relative_offset.x, current_relative_offset.x)
                && unchanged(last_relative_offset.y, current_relative_offset.y)
                && unchanged(last_absolute_offset.x, current_absolute_offset.x)
                && unchanged(last_absolute_offset.y, current_absolute_offset.y)
            {
                return;
            }
        }

        shell.publish(on_scroll(viewport));
        state.last_notified = Some(viewport);
    }
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy)]
pub struct State {
    scroll_area_touched_at: Option<Point>,
    offset_y: Offset,
    y_scroller_grabbed_at: Option<f32>,
    offset_x: Offset,
    x_scroller_grabbed_at: Option<f32>,
    keyboard_modifiers: keyboard::Modifiers,
    last_notified: Option<Viewport>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scroll_area_touched_at: None,
            offset_y: Offset::Absolute(0.0),
            y_scroller_grabbed_at: None,
            offset_x: Offset::Absolute(0.0),
            x_scroller_grabbed_at: None,
            keyboard_modifiers: keyboard::Modifiers::default(),
            last_notified: None,
        }
    }
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, offset: RelativeOffset) {
        State::snap_to(self, offset);
    }

    fn scroll_to(&mut self, offset: AbsoluteOffset) {
        State::scroll_to(self, offset)
    }
}

#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, viewport: f32, content: f32) -> f32 {
        match self {
            Offset::Absolute(absolute) => {
                absolute.min((content - viewport).max(0.0))
            }
            Offset::Relative(percentage) => {
                ((content - viewport) * percentage).max(0.0)
            }
        }
    }

    fn translation(
        self,
        viewport: f32,
        content: f32,
        alignment: Alignment,
    ) -> f32 {
        let offset = self.absolute(viewport, content);

        match alignment {
            Alignment::Start => offset,
            Alignment::End => ((content - viewport).max(0.0) - offset).max(0.0),
        }
    }
}

/// The current [`Viewport`] of the [`Scrollable`].
#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    offset_x: Offset,
    offset_y: Offset,
    bounds: Rectangle,
    content_bounds: Rectangle,
}

impl Viewport {
    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`].
    pub fn absolute_offset(&self) -> AbsoluteOffset {
        let x = self
            .offset_x
            .absolute(self.bounds.width, self.content_bounds.width);
        let y = self
            .offset_y
            .absolute(self.bounds.height, self.content_bounds.height);

        AbsoluteOffset { x, y }
    }

    /// Returns the [`AbsoluteOffset`] of the current [`Viewport`], but with its
    /// alignment reversed.
    ///
    /// This method can be useful to switch the alignment of a [`Scrollable`]
    /// while maintaining its scrolling position.
    pub fn absolute_offset_reversed(&self) -> AbsoluteOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        AbsoluteOffset {
            x: (self.content_bounds.width - self.bounds.width).max(0.0) - x,
            y: (self.content_bounds.height - self.bounds.height).max(0.0) - y,
        }
    }

    /// Returns the [`RelativeOffset`] of the current [`Viewport`].
    pub fn relative_offset(&self) -> RelativeOffset {
        let AbsoluteOffset { x, y } = self.absolute_offset();

        let x = x / (self.content_bounds.width - self.bounds.width);
        let y = y / (self.content_bounds.height - self.bounds.height);

        RelativeOffset { x, y }
    }
}

impl State {
    /// Creates a new [`State`] with the scrollbar(s) at the beginning.
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`Scrollable`] and its contents.
    pub fn scroll(
        &mut self,
        delta: Vector<f32>,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        let horizontal_alignment = direction
            .horizontal()
            .map(|p| p.alignment)
            .unwrap_or_default();

        let vertical_alignment = direction
            .vertical()
            .map(|p| p.alignment)
            .unwrap_or_default();

        let align = |alignment: Alignment, delta: f32| match alignment {
            Alignment::Start => delta,
            Alignment::End => -delta,
        };

        let delta = Vector::new(
            align(horizontal_alignment, delta.x),
            align(vertical_alignment, delta.y),
        );

        if bounds.height < content_bounds.height {
            self.offset_y = Offset::Absolute(
                (self.offset_y.absolute(bounds.height, content_bounds.height)
                    - delta.y)
                    .clamp(0.0, content_bounds.height - bounds.height),
            )
        }

        if bounds.width < content_bounds.width {
            self.offset_x = Offset::Absolute(
                (self.offset_x.absolute(bounds.width, content_bounds.width)
                    - delta.x)
                    .clamp(0.0, content_bounds.width - bounds.width),
            );
        }
    }

    /// Scrolls the [`Scrollable`] to a relative amount along the y axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_y_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset_y = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    /// Scrolls the [`Scrollable`] to a relative amount along the x axis.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn scroll_x_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset_x = Offset::Relative(percentage.clamp(0.0, 1.0));
        self.unsnap(bounds, content_bounds);
    }

    /// Snaps the scroll position to a [`RelativeOffset`].
    pub fn snap_to(&mut self, offset: RelativeOffset) {
        self.offset_x = Offset::Relative(offset.x.clamp(0.0, 1.0));
        self.offset_y = Offset::Relative(offset.y.clamp(0.0, 1.0));
    }

    /// Scroll to the provided [`AbsoluteOffset`].
    pub fn scroll_to(&mut self, offset: AbsoluteOffset) {
        self.offset_x = Offset::Absolute(offset.x.max(0.0));
        self.offset_y = Offset::Absolute(offset.y.max(0.0));
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// [`Scrollable`] and its contents.
    pub fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset_x = Offset::Absolute(
            self.offset_x.absolute(bounds.width, content_bounds.width),
        );
        self.offset_y = Offset::Absolute(
            self.offset_y.absolute(bounds.height, content_bounds.height),
        );
    }

    /// Returns the scrolling translation of the [`State`], given a [`Direction`],
    /// the bounds of the [`Scrollable`] and its contents.
    fn translation(
        &self,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Vector {
        Vector::new(
            if let Some(horizontal) = direction.horizontal() {
                self.offset_x.translation(
                    bounds.width,
                    content_bounds.width,
                    horizontal.alignment,
                )
            } else {
                0.0
            },
            if let Some(vertical) = direction.vertical() {
                self.offset_y.translation(
                    bounds.height,
                    content_bounds.height,
                    vertical.alignment,
                )
            } else {
                0.0
            },
        )
    }

    /// Returns whether any scroller is currently grabbed or not.
    pub fn scrollers_grabbed(&self) -> bool {
        self.x_scroller_grabbed_at.is_some()
            || self.y_scroller_grabbed_at.is_some()
    }
}

#[derive(Debug)]
/// State of both [`Scrollbar`]s.
struct Scrollbars {
    y: Option<internals::Scrollbar>,
    x: Option<internals::Scrollbar>,
}

impl Scrollbars {
    /// Create y and/or x scrollbar(s) if content is overflowing the [`Scrollable`] bounds.
    fn new(
        state: &State,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> Self {
        let translation = state.translation(direction, bounds, content_bounds);

        let show_scrollbar_x = direction
            .horizontal()
            .filter(|_| content_bounds.width > bounds.width);

        let show_scrollbar_y = direction
            .vertical()
            .filter(|_| content_bounds.height > bounds.height);

        let y_scrollbar = if let Some(vertical) = show_scrollbar_y {
            let Properties {
                width,
                margin,
                scroller_width,
                ..
            } = *vertical;

            // Adjust the height of the vertical scrollbar if the horizontal scrollbar
            // is present
            let x_scrollbar_height = show_scrollbar_x
                .map_or(0.0, |h| h.width.max(h.scroller_width) + h.margin);

            let total_scrollbar_width =
                width.max(scroller_width) + 2.0 * margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width,
                y: bounds.y,
                width: total_scrollbar_width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - total_scrollbar_width / 2.0
                    - width / 2.0,
                y: bounds.y,
                width,
                height: (bounds.height - x_scrollbar_height).max(0.0),
            };

            let ratio = bounds.height / content_bounds.height;
            // min height for easier grabbing with super tall content
            let scroller_height = (bounds.height * ratio).max(2.0);
            let scroller_offset = translation.y * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - total_scrollbar_width / 2.0
                    - scroller_width / 2.0,
                y: (scrollbar_bounds.y + scroller_offset - x_scrollbar_height)
                    .max(0.0),
                width: scroller_width,
                height: scroller_height,
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller: internals::Scroller {
                    bounds: scroller_bounds,
                },
                alignment: vertical.alignment,
            })
        } else {
            None
        };

        let x_scrollbar = if let Some(horizontal) = show_scrollbar_x {
            let Properties {
                width,
                margin,
                scroller_width,
                ..
            } = *horizontal;

            // Need to adjust the width of the horizontal scrollbar if the vertical scrollbar
            // is present
            let scrollbar_y_width = show_scrollbar_y
                .map_or(0.0, |v| v.width.max(v.scroller_width) + v.margin);

            let total_scrollbar_height =
                width.max(scroller_width) + 2.0 * margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - total_scrollbar_height,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: total_scrollbar_height,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height
                    - total_scrollbar_height / 2.0
                    - width / 2.0,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: width,
            };

            let ratio = bounds.width / content_bounds.width;
            // min width for easier grabbing with extra wide content
            let scroller_length = (bounds.width * ratio).max(2.0);
            let scroller_offset = translation.x * ratio;

            let scroller_bounds = Rectangle {
                x: (scrollbar_bounds.x + scroller_offset - scrollbar_y_width)
                    .max(0.0),
                y: bounds.y + bounds.height
                    - total_scrollbar_height / 2.0
                    - scroller_width / 2.0,
                width: scroller_length,
                height: scroller_width,
            };

            Some(internals::Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                scroller: internals::Scroller {
                    bounds: scroller_bounds,
                },
                alignment: horizontal.alignment,
            })
        } else {
            None
        };

        Self {
            y: y_scrollbar,
            x: x_scrollbar,
        }
    }

    fn is_mouse_over(&self, cursor: mouse::Cursor) -> (bool, bool) {
        if let Some(cursor_position) = cursor.position() {
            (
                self.y
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
                self.x
                    .as_ref()
                    .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
                    .unwrap_or(false),
            )
        } else {
            (false, false)
        }
    }

    fn grab_y_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.y.and_then(|scrollbar| {
            if scrollbar.total_bounds.contains(cursor_position) {
                Some(if scrollbar.scroller.bounds.contains(cursor_position) {
                    (cursor_position.y - scrollbar.scroller.bounds.y)
                        / scrollbar.scroller.bounds.height
                } else {
                    0.5
                })
            } else {
                None
            }
        })
    }

    fn grab_x_scroller(&self, cursor_position: Point) -> Option<f32> {
        self.x.and_then(|scrollbar| {
            if scrollbar.total_bounds.contains(cursor_position) {
                Some(if scrollbar.scroller.bounds.contains(cursor_position) {
                    (cursor_position.x - scrollbar.scroller.bounds.x)
                        / scrollbar.scroller.bounds.width
                } else {
                    0.5
                })
            } else {
                None
            }
        })
    }

    fn active(&self) -> bool {
        self.y.is_some() || self.x.is_some()
    }
}

pub(super) mod internals {
    use crate::core::{Point, Rectangle};

    use super::Alignment;

    #[derive(Debug, Copy, Clone)]
    pub struct Scrollbar {
        pub total_bounds: Rectangle,
        pub bounds: Rectangle,
        pub scroller: Scroller,
        pub alignment: Alignment,
    }

    impl Scrollbar {
        /// Returns whether the mouse is over the scrollbar or not.
        pub fn is_mouse_over(&self, cursor_position: Point) -> bool {
            self.total_bounds.contains(cursor_position)
        }

        /// Returns the y-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_y(
            &self,
            grabbed_at: f32,
            cursor_position: Point,
        ) -> f32 {
            let percentage = (cursor_position.y
                - self.bounds.y
                - self.scroller.bounds.height * grabbed_at)
                / (self.bounds.height - self.scroller.bounds.height);

            match self.alignment {
                Alignment::Start => percentage,
                Alignment::End => 1.0 - percentage,
            }
        }

        /// Returns the x-axis scrolled percentage from the cursor position.
        pub fn scroll_percentage_x(
            &self,
            grabbed_at: f32,
            cursor_position: Point,
        ) -> f32 {
            let percentage = (cursor_position.x
                - self.bounds.x
                - self.scroller.bounds.width * grabbed_at)
                / (self.bounds.width - self.scroller.bounds.width);

            match self.alignment {
                Alignment::Start => percentage,
                Alignment::End => 1.0 - percentage,
            }
        }
    }

    /// The handle of a [`Scrollbar`].
    #[derive(Debug, Clone, Copy)]
    pub struct Scroller {
        /// The bounds of the [`Scroller`].
        pub bounds: Rectangle,
    }
}
