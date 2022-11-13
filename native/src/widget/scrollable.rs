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

use std::{f32, u32};

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
    content: Element<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(f32) -> Message + 'a>>,
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
    /// Silently enforces a minimum value of 1.
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
    /// It silently enforces a minimum value of 1.
    pub fn scroller_width(mut self, scroller_width: u16) -> Self {
        self.scroller_width = scroller_width.max(1);
        self
    }

    /// Sets a function to call when the [`Scrollable`] is scrolled.
    ///
    /// The function takes the new relative offset of the [`Scrollable`]
    /// (e.g. `0` means top, while `1` means bottom).
    pub fn on_scroll(mut self, f: impl Fn(f32) -> Message + 'a) -> Self {
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

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
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
            u32::MAX,
            |renderer, limits| {
                self.content.as_widget().layout(renderer, limits)
            },
        )
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
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
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
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
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
                let offset = tree
                    .state
                    .downcast_ref::<State>()
                    .offset(bounds, content_bounds);

                overlay.translate(Vector::new(0.0, -(offset as f32)))
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
pub fn snap_to<Message: 'static>(id: Id, percentage: f32) -> Command<Message> {
    Command::widget(operation::scrollable::snap_to(id.0, percentage))
}

/// Computes the layout of a [`Scrollable`].
pub fn layout<Renderer>(
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    max_height: u32,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let limits = limits.max_height(max_height).width(width).height(height);

    let child_limits = layout::Limits::new(
        Size::new(limits.min().width, 0.0),
        Size::new(limits.max().width, f32::INFINITY),
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
    on_scroll: &Option<Box<dyn Fn(f32) -> Message + '_>>,
    update_content: impl FnOnce(
        Event,
        Layout<'_>,
        Point,
        &mut dyn Clipboard,
        &mut Shell<'_, Message>,
    ) -> event::Status,
) -> event::Status {
    let bounds = layout.bounds();
    let is_mouse_over = bounds.contains(cursor_position);

    let content = layout.children().next().unwrap();
    let content_bounds = content.bounds();

    let scrollbar = scrollbar(
        state,
        scrollbar_width,
        scrollbar_margin,
        scroller_width,
        bounds,
        content_bounds,
    );
    let is_mouse_over_scrollbar = scrollbar
        .as_ref()
        .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
        .unwrap_or(false);

    let event_status = {
        let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
            Point::new(
                cursor_position.x,
                cursor_position.y + state.offset(bounds, content_bounds) as f32,
            )
        } else {
            // TODO: Make `cursor_position` an `Option<Point>` so we can encode
            // cursor availability.
            // This will probably happen naturally once we add multi-window
            // support.
            Point::new(cursor_position.x, -1.0)
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

    if is_mouse_over {
        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                match delta {
                    mouse::ScrollDelta::Lines { y, .. } => {
                        // TODO: Configurable speed (?)
                        state.scroll(y * 60.0, bounds, content_bounds);
                    }
                    mouse::ScrollDelta::Pixels { y, .. } => {
                        state.scroll(y, bounds, content_bounds);
                    }
                }

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
                        state.scroll_box_touched_at = Some(cursor_position);
                    }
                    touch::Event::FingerMoved { .. } => {
                        if let Some(scroll_box_touched_at) =
                            state.scroll_box_touched_at
                        {
                            let delta =
                                cursor_position.y - scroll_box_touched_at.y;

                            state.scroll(delta, bounds, content_bounds);

                            state.scroll_box_touched_at = Some(cursor_position);

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
                        state.scroll_box_touched_at = None;
                    }
                }

                return event::Status::Captured;
            }
            _ => {}
        }
    }

    if state.is_scroller_grabbed() {
        match event {
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.scroller_grabbed_at = None;

                return event::Status::Captured;
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if let (Some(scrollbar), Some(scroller_grabbed_at)) =
                    (scrollbar, state.scroller_grabbed_at)
                {
                    state.scroll_to(
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
            }
            _ => {}
        }
    } else if is_mouse_over_scrollbar {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(scrollbar) = scrollbar {
                    if let Some(scroller_grabbed_at) =
                        scrollbar.grab_scroller(cursor_position)
                    {
                        state.scroll_to(
                            scrollbar.scroll_percentage(
                                scroller_grabbed_at,
                                cursor_position,
                            ),
                            bounds,
                            content_bounds,
                        );

                        state.scroller_grabbed_at = Some(scroller_grabbed_at);

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
    cursor_position: Point,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    content_interaction: impl FnOnce(
        Layout<'_>,
        Point,
        &Rectangle,
    ) -> mouse::Interaction,
) -> mouse::Interaction {
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();
    let scrollbar = scrollbar(
        state,
        scrollbar_width,
        scrollbar_margin,
        scroller_width,
        bounds,
        content_bounds,
    );

    let is_mouse_over = bounds.contains(cursor_position);
    let is_mouse_over_scrollbar = scrollbar
        .as_ref()
        .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
        .unwrap_or(false);

    if is_mouse_over_scrollbar || state.is_scroller_grabbed() {
        mouse::Interaction::Idle
    } else {
        let offset = state.offset(bounds, content_bounds);

        let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
            Point::new(cursor_position.x, cursor_position.y + offset as f32)
        } else {
            Point::new(cursor_position.x, -1.0)
        };

        content_interaction(
            content_layout,
            cursor_position,
            &Rectangle {
                y: bounds.y + offset as f32,
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
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    style: &<Renderer::Theme as StyleSheet>::Style,
    draw_content: impl FnOnce(&mut Renderer, Layout<'_>, Point, &Rectangle),
) where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();
    let offset = state.offset(bounds, content_bounds);
    let scrollbar = scrollbar(
        state,
        scrollbar_width,
        scrollbar_margin,
        scroller_width,
        bounds,
        content_bounds,
    );

    let is_mouse_over = bounds.contains(cursor_position);
    let is_mouse_over_scrollbar = scrollbar
        .as_ref()
        .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
        .unwrap_or(false);

    let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
        Point::new(cursor_position.x, cursor_position.y + offset as f32)
    } else {
        Point::new(cursor_position.x, -1.0)
    };

    if let Some(scrollbar) = scrollbar {
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(
                Vector::new(0.0, -(offset as f32)),
                |renderer| {
                    draw_content(
                        renderer,
                        content_layout,
                        cursor_position,
                        &Rectangle {
                            y: bounds.y + offset as f32,
                            ..bounds
                        },
                    );
                },
            );
        });

        let style = if state.is_scroller_grabbed() {
            theme.dragging(style)
        } else if is_mouse_over_scrollbar {
            theme.hovered(style)
        } else {
            theme.active(style)
        };

        let is_scrollbar_visible =
            style.background.is_some() || style.border_width > 0.0;

        renderer.with_layer(
            Rectangle {
                width: bounds.width + 2.0,
                height: bounds.height + 2.0,
                ..bounds
            },
            |renderer| {
                if is_scrollbar_visible {
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

                if is_mouse_over
                    || state.is_scroller_grabbed()
                    || is_scrollbar_visible
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
            },
        );
    } else {
        draw_content(
            renderer,
            content_layout,
            cursor_position,
            &Rectangle {
                y: bounds.y + offset as f32,
                ..bounds
            },
        );
    }
}

fn scrollbar(
    state: &State,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    bounds: Rectangle,
    content_bounds: Rectangle,
) -> Option<Scrollbar> {
    let offset = state.offset(bounds, content_bounds);

    if content_bounds.height > bounds.height {
        let outer_width =
            scrollbar_width.max(scroller_width) + 2 * scrollbar_margin;

        let outer_bounds = Rectangle {
            x: bounds.x + bounds.width - outer_width as f32,
            y: bounds.y,
            width: outer_width as f32,
            height: bounds.height,
        };

        let scrollbar_bounds = Rectangle {
            x: bounds.x + bounds.width
                - f32::from(outer_width / 2 + scrollbar_width / 2),
            y: bounds.y,
            width: scrollbar_width as f32,
            height: bounds.height,
        };

        let ratio = bounds.height / content_bounds.height;
        let scroller_height = bounds.height * ratio;
        let y_offset = offset as f32 * ratio;

        let scroller_bounds = Rectangle {
            x: bounds.x + bounds.width
                - f32::from(outer_width / 2 + scroller_width / 2),
            y: scrollbar_bounds.y + y_offset,
            width: scroller_width as f32,
            height: scroller_height,
        };

        Some(Scrollbar {
            outer_bounds,
            bounds: scrollbar_bounds,
            scroller: Scroller {
                bounds: scroller_bounds,
            },
        })
    } else {
        None
    }
}

fn notify_on_scroll<Message>(
    state: &State,
    on_scroll: &Option<Box<dyn Fn(f32) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) {
    if content_bounds.height <= bounds.height {
        return;
    }

    if let Some(on_scroll) = on_scroll {
        shell.publish(on_scroll(
            state.offset.absolute(bounds, content_bounds)
                / (content_bounds.height - bounds.height),
        ));
    }
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy)]
pub struct State {
    scroller_grabbed_at: Option<f32>,
    scroll_box_touched_at: Option<Point>,
    offset: Offset,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scroller_grabbed_at: None,
            scroll_box_touched_at: None,
            offset: Offset::Absolute(0.0),
        }
    }
}

impl operation::Scrollable for State {
    fn snap_to(&mut self, percentage: f32) {
        State::snap_to(self, percentage);
    }
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(self, bounds: Rectangle, content_bounds: Rectangle) -> f32 {
        match self {
            Self::Absolute(absolute) => {
                let hidden_content =
                    (content_bounds.height - bounds.height).max(0.0);

                absolute.min(hidden_content)
            }
            Self::Relative(percentage) => {
                ((content_bounds.height - bounds.height) * percentage).max(0.0)
            }
        }
    }
}

impl State {
    /// Creates a new [`State`] with the scrollbar located at the top.
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`Scrollable`] and its contents.
    pub fn scroll(
        &mut self,
        delta_y: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        if bounds.height >= content_bounds.height {
            return;
        }

        self.offset = Offset::Absolute(
            (self.offset.absolute(bounds, content_bounds) - delta_y)
                .max(0.0)
                .min((content_bounds.height - bounds.height) as f32),
        );
    }

    /// Scrolls the [`Scrollable`] to a relative amount.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    pub fn scroll_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.snap_to(percentage);
        self.unsnap(bounds, content_bounds);
    }

    /// Snaps the scroll position to a relative amount.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    pub fn snap_to(&mut self, percentage: f32) {
        self.offset = Offset::Relative(percentage.max(0.0).min(1.0));
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// [`Scrollable`] and its contents.
    pub fn unsnap(&mut self, bounds: Rectangle, content_bounds: Rectangle) {
        self.offset =
            Offset::Absolute(self.offset.absolute(bounds, content_bounds));
    }

    /// Returns the current scrolling offset of the [`State`], given the bounds
    /// of the [`Scrollable`] and its contents.
    pub fn offset(&self, bounds: Rectangle, content_bounds: Rectangle) -> u32 {
        self.offset.absolute(bounds, content_bounds) as u32
    }

    /// Returns whether the scroller is currently grabbed or not.
    pub fn is_scroller_grabbed(&self) -> bool {
        self.scroller_grabbed_at.is_some()
    }

    /// Returns whether the scroll box is currently touched or not.
    pub fn is_scroll_box_touched(&self) -> bool {
        self.scroll_box_touched_at.is_some()
    }
}

/// The scrollbar of a [`Scrollable`].
#[derive(Debug)]
struct Scrollbar {
    /// The outer bounds of the scrollable, including the [`Scrollbar`] and
    /// [`Scroller`].
    outer_bounds: Rectangle,

    /// The bounds of the [`Scrollbar`].
    bounds: Rectangle,

    /// The bounds of the [`Scroller`].
    scroller: Scroller,
}

impl Scrollbar {
    fn is_mouse_over(&self, cursor_position: Point) -> bool {
        self.outer_bounds.contains(cursor_position)
    }

    fn grab_scroller(&self, cursor_position: Point) -> Option<f32> {
        if self.outer_bounds.contains(cursor_position) {
            Some(if self.scroller.bounds.contains(cursor_position) {
                (cursor_position.y - self.scroller.bounds.y)
                    / self.scroller.bounds.height
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
        (cursor_position.y
            - self.bounds.y
            - self.scroller.bounds.height * grabbed_at)
            / (self.bounds.height - self.scroller.bounds.height)
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
struct Scroller {
    /// The bounds of the [`Scroller`].
    bounds: Rectangle,
}
