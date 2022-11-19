//! Navigate an endless amount of content with a scrollbar.
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::touch;
use crate::widget;
use crate::widget::operation::{self, Operation};
use crate::widget::tree::{self, Tree};
use crate::{
    Background, Clipboard, Color, Command, Element, Layout, Length, Point,
    Rectangle, Shell, Size, Vector, Widget,
};

pub use iced_style::scrollable::StyleSheet;

pub mod style {
    //! The styles of a [`Scrollable`].
    //!
    //! [`Scrollable`]: crate::widget::Scrollable
    pub use iced_style::scrollable::{Scrollbar, Scroller};
}

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    id: Option<Id>,
    height: Length,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    scroll_horizontal: Option<Horizontal>,
    content: Element<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(Vector<f32>) -> Message + 'a>>,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Message, Renderer> Scrollable<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Scrollable`].
    pub fn new(content: impl Into<Element<'a, Message, Renderer>>) -> Self {
        Scrollable {
            id: None,
            height: Length::Shrink,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            scroll_horizontal: None,
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

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the scrollbar width of the [`Scrollable`] .
    /// Silently enforces a minimum width of 1.
    pub fn scrollbar_width(mut self, scrollbar_width: u16) -> Self {
        self.scrollbar_width = scrollbar_width.max(1);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollable`] .
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller width of the [`Scrollable`] .
    ///
    /// It silently enforces a minimum width of 1.
    pub fn scroller_width(mut self, scroller_width: u16) -> Self {
        self.scroller_width = scroller_width.max(1);
        self
    }

    /// Allow scrolling in a horizontal direction within the [`Scrollable`] .
    pub fn horizontal_scroll(mut self, horizontal: Horizontal) -> Self {
        self.scroll_horizontal = Some(horizontal);
        self
    }

    /// Sets a function to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the new relative x & y offset of the [`Scrollable`]
    /// (e.g. `0` means beginning, while `1` means end).
    pub fn on_scroll(
        mut self,
        f: impl Fn(Vector<f32>) -> Message + 'a,
    ) -> Self {
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

/// Properties of a horizontal scrollbar within a [`Scrollable`].
#[derive(Debug)]
pub struct Horizontal {
    scrollbar_height: u16,
    scrollbar_margin: u16,
    scroller_height: u16,
}

impl Default for Horizontal {
    fn default() -> Self {
        Self {
            scrollbar_height: 10,
            scrollbar_margin: 0,
            scroller_height: 10,
        }
    }
}

impl Horizontal {
    /// Creates a new [`Horizontal`] for use in a [`Scrollable`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`Horizontal`] scrollbar height of the [`Scrollable`] .
    /// Silently enforces a minimum height of 1.
    pub fn scrollbar_height(mut self, scrollbar_height: u16) -> Self {
        self.scrollbar_height = scrollbar_height.max(1);
        self
    }

    /// Sets the [`Horizontal`] scrollbar margin of the [`Scrollable`] .
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller height of the [`Horizontal`] scrollbar of the [`Scrollable`] .
    /// Silently enforces a minimum height of 1.
    pub fn scroller_height(mut self, scroller_height: u16) -> Self {
        self.scroller_height = scroller_height.max(1);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn width(&self) -> Length {
        self.content.as_widget().width()
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
            Widget::<Message, Renderer>::width(self),
            self.height,
            self.scroll_horizontal.is_some(),
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
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
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            tree.state.downcast_ref::<State>(),
            renderer,
            theme,
            layout,
            cursor_position,
            &self.style,
            |renderer, layout, cursor_position, viewport| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout,
                    cursor_position,
                    viewport,
                )
            },
        )
    }

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

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<State>();

        operation.scrollable(state, self.id.as_ref().map(|id| &id.0));

        operation.container(None, &mut |operation| {
            self.content.as_widget().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                operation,
            );
        });
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
        update(
            tree.state.downcast_mut::<State>(),
            event,
            layout,
            cursor_position,
            clipboard,
            shell,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
            self.scroll_horizontal.as_ref(),
            &self.on_scroll,
            |event, layout, cursor_position, clipboard, shell| {
                self.content.as_widget_mut().on_event(
                    &mut tree.children[0],
                    event,
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    shell,
                )
            },
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(
            tree.state.downcast_ref::<State>(),
            layout,
            cursor_position,
            |layout, cursor_position, viewport| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            },
        )
    }

    fn overlay<'b>(
        &'b self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer>> {
        self.content
            .as_widget()
            .overlay(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
            )
            .map(|overlay| {
                let bounds = layout.bounds();
                let content_layout = layout.children().next().unwrap();
                let content_bounds = content_layout.bounds();
                let (offset_x, offset_y) = tree
                    .state
                    .downcast_ref::<State>()
                    .offset(bounds, content_bounds);

                overlay.translate(Vector::new(
                    -(offset_x as f32),
                    -(offset_y as f32),
                ))
            })
    }
}

impl<'a, Message, Renderer> From<Scrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Message: 'a,
    Renderer: 'a + crate::Renderer,
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
/// to the provided `percentage`.
pub fn snap_to<Message: 'static>(
    id: Id,
    percentage: Vector<f32>,
) -> Command<Message> {
    Command::widget(operation::scrollable::snap_to(id.0, percentage))
}

/// Computes the layout of a [`Scrollable`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    horizontal_enabled: bool,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits
        .max_height(u32::MAX)
        .max_width(if horizontal_enabled {
            u32::MAX
        } else {
            limits.max().width as u32
        })
        .width(width)
        .height(height);

    let child_limits = layout::Limits::new(
        Size::new(limits.min().width, 0.0),
        Size::new(
            if horizontal_enabled {
                f32::INFINITY
            } else {
                limits.max().width
            },
            f32::MAX,
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
    cursor_position: Point,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    horizontal: Option<&Horizontal>,
    on_scroll: &Option<Box<dyn Fn(Vector<f32>) -> Message + '_>>,
    update_content: impl FnOnce(
        Event,
        Layout<'_>,
        Point,
        &mut dyn Clipboard,
        &mut Shell<'_, Message>,
    ) -> event::Status,
) -> event::Status {
    let bounds = layout.bounds();
    let mouse_over_scrollable = bounds.contains(cursor_position);

    let content = layout.children().next().unwrap();
    let content_bounds = content.bounds();

    state.create_scrollbars_maybe(
        horizontal,
        scrollbar_width,
        scrollbar_margin,
        scroller_width,
        bounds,
        content_bounds,
    );

    let (mouse_over_x_scrollbar, mouse_over_y_scrollbar) =
        state.mouse_over_scrollbars(cursor_position);

    let event_status = {
        let cursor_position = if mouse_over_scrollable
            && !(mouse_over_y_scrollbar || mouse_over_x_scrollbar)
        {
            let (offset_x, offset_y) = state.offset(bounds, content_bounds);

            Point::new(
                cursor_position.x + offset_x as f32,
                cursor_position.y + offset_y as f32,
            )
        } else {
            // TODO: Make `cursor_position` an `Option<Point>` so we can encode
            // cursor availability.
            // This will probably happen naturally once we add multi-window
            // support.
            Point::new(-1.0, -1.0)
        };

        update_content(
            event.clone(),
            content,
            cursor_position,
            clipboard,
            shell,
        )
    };

    if let event::Status::Captured = event_status {
        return event::Status::Captured;
    }

    if mouse_over_scrollable {
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let delta = match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        // TODO: Configurable speed/friction (?)
                        Vector::new(x * 60.0, y * 60.0)
                    }
                    mouse::ScrollDelta::Pixels { x, y } => Vector::new(x, y),
                };

                state.scroll(delta, bounds, content_bounds);

                notify_on_scroll(
                    state,
                    on_scroll,
                    bounds,
                    content_bounds,
                    shell,
                );

                return event::Status::Captured;
            }
            Event::Touch(event) => {
                match event {
                    touch::Event::FingerPressed { .. } => {
                        state.scroll_area_touched_at = Some(cursor_position);
                    }
                    touch::Event::FingerMoved { .. } => {
                        if let Some(scroll_box_touched_at) =
                            state.scroll_area_touched_at
                        {
                            let delta = Vector::new(
                                cursor_position.x - scroll_box_touched_at.x,
                                cursor_position.y - scroll_box_touched_at.y,
                            );

                            state.scroll(delta, bounds, content_bounds);

                            state.scroll_area_touched_at =
                                Some(cursor_position);

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
    }

    if let Some(scrollbar) = &mut state.scrollbar_y {
        if let Some(scroller_grabbed_at) = scrollbar.scroller.grabbed_at {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    scrollbar.scroller.grabbed_at = None;

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    scrollbar.scroll_to(
                        scrollbar.scroll_percentage(
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
                _ => {}
            }
        } else if scrollbar.is_mouse_over(cursor_position) {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(scroller_grabbed_at) =
                        scrollbar.grab_scroller(cursor_position)
                    {
                        scrollbar.scroll_to(
                            scrollbar.scroll_percentage(
                                scroller_grabbed_at,
                                cursor_position,
                            ),
                            bounds,
                            content_bounds,
                        );

                        scrollbar.scroller.grabbed_at =
                            Some(scroller_grabbed_at);

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
    }

    if let Some(scrollbar) = &mut state.scrollbar_x {
        if let Some(scroller_grabbed_at) = scrollbar.scroller.grabbed_at {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    scrollbar.scroller.grabbed_at = None;

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    scrollbar.scroll_to(
                        scrollbar.scroll_percentage(
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
                _ => {}
            }
        } else if scrollbar.is_mouse_over(cursor_position) {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(scroller_grabbed_at) =
                        scrollbar.grab_scroller(cursor_position)
                    {
                        scrollbar.scroll_to(
                            scrollbar.scroll_percentage(
                                scroller_grabbed_at,
                                cursor_position,
                            ),
                            bounds,
                            content_bounds,
                        );

                        scrollbar.scroller.grabbed_at =
                            Some(scroller_grabbed_at);

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
    }

    event::Status::Ignored
}

/// Computes the current [`mouse::Interaction`] of a [`Scrollable`].
pub fn mouse_interaction(
    state: &State,
    layout: Layout<'_>,
    cursor_position: Point,
    content_interaction: impl FnOnce(
        Layout<'_>,
        Point,
        &Rectangle,
    ) -> mouse::Interaction,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let mouse_over_scrollable = bounds.contains(cursor_position);

    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let (mouse_over_x_scrollbar, mouse_over_y_scrollbar) =
        state.mouse_over_scrollbars(cursor_position);

    if (mouse_over_x_scrollbar || mouse_over_y_scrollbar)
        || state.scrollers_grabbed()
    {
        mouse::Interaction::Idle
    } else {
        let (offset_x, offset_y) = state.offset(bounds, content_bounds);

        let cursor_position = if mouse_over_scrollable
            && !(mouse_over_y_scrollbar || mouse_over_x_scrollbar)
        {
            Point::new(
                cursor_position.x + offset_x as f32,
                cursor_position.y + offset_y as f32,
            )
        } else {
            Point::new(-1.0, -1.0)
        };

        content_interaction(
            content_layout,
            cursor_position,
            &Rectangle {
                y: bounds.y + offset_y as f32,
                x: bounds.x + offset_x as f32,
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
    cursor_position: Point,
    style: &<Renderer::Theme as StyleSheet>::Style,
    draw_content: impl FnOnce(&mut Renderer, Layout<'_>, Point, &Rectangle),
) where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();

    let (offset_x, offset_y) = state.offset(bounds, content_bounds);

    let mouse_over_scrollable = bounds.contains(cursor_position);

    let (mouse_over_x_scrollbar, mouse_over_y_scrollbar) =
        state.mouse_over_scrollbars(cursor_position);

    let cursor_position = if mouse_over_scrollable
        && !(mouse_over_x_scrollbar || mouse_over_y_scrollbar)
    {
        Point::new(
            cursor_position.x + offset_x as f32,
            cursor_position.y + offset_y as f32,
        )
    } else {
        Point::new(-1.0, -1.0)
    };

    // Draw inner content
    if state.scrollbar_y.is_some() || state.scrollbar_x.is_some() {
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(
                Vector::new(-(offset_x as f32), -(offset_y as f32)),
                |renderer| {
                    draw_content(
                        renderer,
                        content_layout,
                        cursor_position,
                        &Rectangle {
                            y: bounds.y + offset_y as f32,
                            x: bounds.x + offset_x as f32,
                            ..bounds
                        },
                    );
                },
            );
        });

        let draw_scrollbar =
            |renderer: &mut Renderer, scrollbar: Option<&Scrollbar>| {
                if let Some(scrollbar) = scrollbar {
                    let style = match scrollbar.direction {
                        Direction::Vertical => {
                            if scrollbar.scroller.grabbed_at.is_some() {
                                theme.dragging(style)
                            } else if mouse_over_y_scrollbar {
                                theme.hovered(style)
                            } else {
                                theme.active(style)
                            }
                        }
                        Direction::Horizontal => {
                            if scrollbar.scroller.grabbed_at.is_some() {
                                theme.dragging_horizontal(style)
                            } else if mouse_over_x_scrollbar {
                                theme.hovered_horizontal(style)
                            } else {
                                theme.active_horizontal(style)
                            }
                        }
                    };

                    //track
                    if style.background.is_some()
                        || (style.border_color != Color::TRANSPARENT
                            && style.border_width > 0.0)
                    {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: scrollbar.bounds,
                                border_radius: style.border_radius,
                                border_width: style.border_width,
                                border_color: style.border_color,
                            },
                            style.background.unwrap_or(Background::Color(
                                Color::TRANSPARENT,
                            )),
                        );
                    }

                    //thumb
                    if style.scroller.color != Color::TRANSPARENT
                        || (style.scroller.border_color != Color::TRANSPARENT
                            && style.scroller.border_width > 0.0)
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
                }
            };

        renderer.with_layer(
            Rectangle {
                width: bounds.width + 2.0,
                height: bounds.height + 2.0,
                ..bounds
            },
            |renderer| {
                draw_scrollbar(renderer, state.scrollbar_y.as_ref());
                draw_scrollbar(renderer, state.scrollbar_x.as_ref());
            },
        );
    } else {
        draw_content(
            renderer,
            content_layout,
            cursor_position,
            &Rectangle {
                x: bounds.x + offset_x as f32,
                y: bounds.y + offset_y as f32,
                ..bounds
            },
        );
    }
}

fn notify_on_scroll<Message>(
    state: &State,
    on_scroll: &Option<Box<dyn Fn(Vector<f32>) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) {
    if let Some(on_scroll) = on_scroll {
        let delta_x = if content_bounds.width <= bounds.width {
            0.0
        } else {
            state.scrollbar_x.map_or(0.0, |scrollbar| {
                scrollbar.offset.absolute(
                    Direction::Horizontal,
                    bounds,
                    content_bounds,
                ) / (content_bounds.width - bounds.width)
            })
        };

        let delta_y = if content_bounds.height <= bounds.height {
            0.0
        } else {
            state.scrollbar_y.map_or(0.0, |scrollbar| {
                scrollbar.offset.absolute(
                    Direction::Vertical,
                    bounds,
                    content_bounds,
                ) / (content_bounds.height - bounds.height)
            })
        };

        shell.publish(on_scroll(Vector::new(delta_x, delta_y)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The direction of the [`Scrollable`].
pub enum Direction {
    /// X or horizontal
    Horizontal,
    /// Y or vertical
    Vertical,
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    scroll_area_touched_at: Option<Point>,
    scrollbar_x: Option<Scrollbar>,
    scrollbar_y: Option<Scrollbar>,
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, percentage: Vector<f32>) {
        if let Some(scrollbar) = &mut self.scrollbar_y {
            scrollbar.snap_to(percentage.y)
        }
        if let Some(scrollbar) = &mut self.scrollbar_x {
            scrollbar.snap_to(percentage.x)
        }
    }
}

impl State {
    /// Creates a new [`State`].
    pub fn new() -> Self {
        State::default()
    }

    /// Create y or x scrollbars if content is overflowing the [`Scrollable`] bounds.
    pub fn create_scrollbars_maybe(
        &mut self,
        horizontal: Option<&Horizontal>,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        let show_scrollbar_x = horizontal.and_then(|h| {
            if content_bounds.width > bounds.width {
                Some(h)
            } else {
                None
            }
        });

        self.scrollbar_y = if content_bounds.height > bounds.height {
            let (offset_y, scroller_grabbed) =
                if let Some(scrollbar) = &self.scrollbar_y {
                    (
                        scrollbar.offset.absolute(
                            scrollbar.direction,
                            bounds,
                            content_bounds,
                        ),
                        scrollbar.scroller.grabbed_at,
                    )
                } else {
                    (0.0, None)
                };

            // Need to adjust the height of the vertical scrollbar if the horizontal scrollbar
            // is present
            let scrollbar_x_height = show_scrollbar_x.map_or(0.0, |h| {
                (h.scrollbar_height.max(h.scroller_height) + h.scrollbar_margin)
                    as f32
            });

            let total_scrollbar_width =
                scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width - total_scrollbar_width as f32,
                y: bounds.y,
                width: total_scrollbar_width as f32,
                height: (bounds.height - scrollbar_x_height).max(0.0),
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(
                        total_scrollbar_width / 2 + scrollbar_width / 2,
                    ),
                y: bounds.y,
                width: scrollbar_width as f32,
                height: (bounds.height - scrollbar_x_height).max(0.0),
            };

            let ratio = bounds.height / content_bounds.height;
            // min height for easier grabbing with super tall content
            let scroller_height = (bounds.height * ratio).max(2.0);
            let scroller_offset = offset_y as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: bounds.x + bounds.width
                    - f32::from(total_scrollbar_width / 2 + scroller_width / 2),
                y: (scrollbar_bounds.y + scroller_offset - scrollbar_x_height)
                    .max(0.0),
                width: scroller_width as f32,
                height: scroller_height,
            };

            Some(Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                direction: Direction::Vertical,
                scroller: Scroller {
                    bounds: scroller_bounds,
                    grabbed_at: scroller_grabbed,
                },
                offset: Offset::Absolute(offset_y),
            })
        } else {
            None
        };

        self.scrollbar_x = if let Some(horizontal) = show_scrollbar_x {
            let (offset_x, scroller_grabbed) =
                if let Some(scrollbar) = &self.scrollbar_x {
                    (
                        scrollbar.offset.absolute(
                            scrollbar.direction,
                            bounds,
                            content_bounds,
                        ),
                        scrollbar.scroller.grabbed_at,
                    )
                } else {
                    (0.0, None)
                };

            // Need to adjust the width of the horizontal scrollbar if the vertical scrollbar
            // is present
            let scrollbar_y_width = self.scrollbar_y.map_or(0.0, |_| {
                (scrollbar_width.max(scroller_width) + scrollbar_margin) as f32
            });

            let total_scrollbar_height =
                horizontal.scrollbar_height.max(horizontal.scroller_height)
                    + 2 * horizontal.scrollbar_margin;

            // Total bounds of the scrollbar + margin + scroller width
            let total_scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height - total_scrollbar_height as f32,
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: total_scrollbar_height as f32,
            };

            // Bounds of just the scrollbar
            let scrollbar_bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + bounds.height
                    - f32::from(
                        total_scrollbar_height / 2
                            + horizontal.scrollbar_height / 2,
                    ),
                width: (bounds.width - scrollbar_y_width).max(0.0),
                height: horizontal.scrollbar_height as f32,
            };

            let ratio = bounds.width / content_bounds.width;
            // min width for easier grabbing with extra wide content
            let scroller_width = (bounds.width * ratio).max(2.0);
            let scroller_offset = offset_x as f32 * ratio;

            let scroller_bounds = Rectangle {
                x: (scrollbar_bounds.x + scroller_offset - scrollbar_y_width)
                    .max(0.0),
                y: bounds.y + bounds.height
                    - f32::from(
                        total_scrollbar_height / 2
                            + horizontal.scroller_height / 2,
                    ),
                width: scroller_width,
                height: horizontal.scroller_height as f32,
            };

            Some(Scrollbar {
                total_bounds: total_scrollbar_bounds,
                bounds: scrollbar_bounds,
                direction: Direction::Horizontal,
                scroller: Scroller {
                    bounds: scroller_bounds,
                    grabbed_at: scroller_grabbed,
                },
                offset: Offset::Absolute(offset_x),
            })
        } else {
            None
        };
    }

    /// Returns whether the mouse is within the bounds of each scrollbar.
    fn mouse_over_scrollbars(&self, cursor_position: Point) -> (bool, bool) {
        (
            self.scrollbar_x.map_or(false, |scrollbar| {
                scrollbar.is_mouse_over(cursor_position)
            }),
            self.scrollbar_y.map_or(false, |scrollbar| {
                scrollbar.is_mouse_over(cursor_position)
            }),
        )
    }

    /// Returns whether the scroller for either scrollbar is currently grabbed.
    fn scrollers_grabbed(&self) -> bool {
        self.scrollbar_x
            .map_or(false, |scrollbar| scrollbar.scroller.grabbed_at.is_some())
            || self.scrollbar_y.map_or(false, |scrollbar| {
                scrollbar.scroller.grabbed_at.is_some()
            })
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`Scrollable`] and its contents.
    pub fn scroll(
        &mut self,
        delta: Vector<f32>,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        if delta.x != 0.0 && bounds.width < content_bounds.width {
            if let Some(scrollbar) = &mut self.scrollbar_x {
                scrollbar.offset = Offset::Absolute(
                    (scrollbar.offset.absolute(
                        Direction::Horizontal,
                        bounds,
                        content_bounds,
                    ) - delta.x)
                        .max(0.0)
                        .min((content_bounds.width - bounds.width) as f32),
                );
            }
        }

        if delta.y != 0.0 && bounds.height < content_bounds.height {
            if let Some(scrollbar) = &mut self.scrollbar_y {
                scrollbar.offset = Offset::Absolute(
                    (scrollbar.offset.absolute(
                        Direction::Vertical,
                        bounds,
                        content_bounds,
                    ) - delta.y)
                        .max(0.0)
                        .min((content_bounds.height - bounds.height) as f32),
                )
            }
        }
    }

    /// Returns the current x & y scrolling offset of the [`State`], given the bounds
    /// of the [`Scrollable`] and its contents.
    pub fn offset(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> (f32, f32) {
        (
            self.scrollbar_x.map_or(0.0, |scrollbar| {
                scrollbar.offset.absolute(
                    Direction::Horizontal,
                    bounds,
                    content_bounds,
                )
            }),
            self.scrollbar_y.map_or(0.0, |scrollbar| {
                scrollbar.offset.absolute(
                    Direction::Vertical,
                    bounds,
                    content_bounds,
                )
            }),
        )
    }
}

/// The scrollbar of a [`Scrollable`].
#[derive(Debug, Copy, Clone)]
struct Scrollbar {
    /// The total bounds of the [`Scrollbar`], including the scrollbar, the scroller,
    /// and the scrollbar margin.
    total_bounds: Rectangle,

    /// The bounds of just the [`Scrollbar`].
    bounds: Rectangle,

    /// The direction of the [`Scrollbar`].
    direction: Direction,

    /// The state of this scrollbar's [`Scroller`].
    scroller: Scroller,

    /// The current offset of the [`Scrollbar`].
    offset: Offset,
}

impl Scrollbar {
    /// Snaps the scroll position to a relative amount.
    ///
    /// `0` represents scrollbar at the beginning, while `1` represents scrollbar at
    /// the end.
    pub fn snap_to(&mut self, percentage: f32) {
        self.offset = Offset::Relative(percentage.max(0.0).min(1.0));
    }

    /// Unsnaps the current scroll position if snapped, given the bounds of the [`Scrollable`]
    /// and its contents.
    pub fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset = Offset::Absolute(self.offset.absolute(
            self.direction,
            bounds,
            content_bounds,
        ));
    }

    /// Scrolls the [`Scrollbar`] to a certain percentage.
    fn scroll_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.snap_to(percentage);
        self.unsnap(bounds, content_bounds);
    }

    /// Returns whether the mouse is over the scrollbar or not.
    fn is_mouse_over(&self, cursor_position: Point) -> bool {
        self.total_bounds.contains(cursor_position)
    }

    fn grab_scroller(&self, cursor_position: Point) -> Option<f32> {
        if self.total_bounds.contains(cursor_position) {
            Some(if self.scroller.bounds.contains(cursor_position) {
                match self.direction {
                    Direction::Vertical => {
                        (cursor_position.y - self.scroller.bounds.y)
                            / self.scroller.bounds.height
                    }
                    Direction::Horizontal => {
                        (cursor_position.x - self.scroller.bounds.x)
                            / self.scroller.bounds.width
                    }
                }
            } else {
                0.5
            })
        } else {
            None
        }
    }

    fn scroll_percentage(
        &self,
        grabbed_at: f32,
        cursor_position: Point,
    ) -> f32 {
        match self.direction {
            Direction::Vertical => {
                (cursor_position.y
                    - self.bounds.y
                    - self.scroller.bounds.height * grabbed_at)
                    / (self.bounds.height - self.scroller.bounds.height)
            }
            Direction::Horizontal => {
                (cursor_position.x
                    - self.bounds.x
                    - self.scroller.bounds.width * grabbed_at)
                    / (self.bounds.width - self.scroller.bounds.width)
            }
        }
    }
}

/// The directional offset of a [`Scrollable`].
#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(
        self,
        direction: Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> f32 {
        match self {
            Self::Absolute(absolute) => match direction {
                Direction::Horizontal => {
                    absolute.min((content_bounds.width - bounds.width).max(0.0))
                }
                Direction::Vertical => absolute
                    .min((content_bounds.height - bounds.height).max(0.0)),
            },
            Self::Relative(percentage) => match direction {
                Direction::Horizontal => {
                    ((content_bounds.width - bounds.width) * percentage)
                        .max(0.0)
                }
                Direction::Vertical => {
                    ((content_bounds.height - bounds.height) * percentage)
                        .max(0.0)
                }
            },
        }
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
struct Scroller {
    /// The bounds of the [`Scroller`].
    bounds: Rectangle,

    /// Whether or not the scroller is currently grabbed.
    grabbed_at: Option<f32>,
}
