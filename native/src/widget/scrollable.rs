//! Navigate an endless amount of content with a scrollbar.
mod horizontal;
mod vertical;

use crate::widget::{operation, Operation, Tree};
use crate::{
    event, layout, mouse, overlay, renderer, touch, widget, Clipboard, Command,
    Element, Event, Layout, Shell,
};
pub use horizontal::Horizontal;
use iced_core::{Background, Color, Length, Point, Rectangle, Size, Vector};
pub use iced_style::scrollable::StyleSheet;
pub use vertical::Vertical;

pub mod style {
    //! The styles of a [`Scrollable`].
    //!
    //! [`Scrollable`]: crate::widget::Scrollable
    pub use iced_style::scrollable::{Scrollbar, Scroller};
}

/// Creates a new [`Horizontal`] scrollable.
pub fn horizontal<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Horizontal<'a, Message, Renderer>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    Horizontal::new(content)
}

/// Creates a new [`Vertical`] scrollable.
pub fn vertical<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
) -> Vertical<'a, Message, Renderer>
    where
        Renderer: crate::Renderer,
        Renderer::Theme: StyleSheet,
{
    Vertical::new(content)
}

/// The identifier of a scrollable.
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

/// Produces a [`Command`] that snaps the scrollable with the given [`Id`]
/// to the provided `percentage`.
pub fn snap_to<Message: 'static>(id: Id, percentage: f32) -> Command<Message> {
    Command::widget(operation::scrollable::snap_to(id.0, percentage))
}

#[derive(Debug)]
/// The direction of the scrollable.
pub enum Direction {
    /// Vertical
    Vertical,
    /// Horizontal
    Horizontal,
}

/// The local state of a scrollable.
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

/// The local offset of a scrollable.
#[derive(Debug, Clone, Copy)]
enum Offset {
    Absolute(f32),
    Relative(f32),
}

impl Offset {
    fn absolute(
        self,
        direction: &Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> f32 {
        match self {
            Self::Absolute(absolute) => {
                let hidden_content = match direction {
                    Direction::Vertical => {
                        (content_bounds.height - bounds.height).max(0.0)
                    }
                    Direction::Horizontal => {
                        (content_bounds.width - bounds.width).max(0.0)
                    }
                };

                absolute.min(hidden_content)
            }
            Self::Relative(percentage) => match direction {
                Direction::Vertical => {
                    ((content_bounds.height - bounds.height) * percentage)
                        .max(0.0)
                }
                Direction::Horizontal => {
                    ((content_bounds.width - bounds.width) * percentage)
                        .max(0.0)
                }
            },
        }
    }
}

impl State {
    /// Creates a new [`State`] with the scrollbar located at the top.
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the scrollable and its contents.
    pub fn scroll(
        &mut self,
        direction: &Direction,
        delta: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        match direction {
            Direction::Vertical => {
                if bounds.height >= content_bounds.height {
                    return;
                }

                self.offset = Offset::Absolute(
                    (self.offset.absolute(direction, bounds, content_bounds)
                        - delta)
                        .max(0.0)
                        .min((content_bounds.height - bounds.height) as f32),
                )
            }
            Direction::Horizontal => {
                if bounds.width >= content_bounds.width {
                    return;
                }

                self.offset = Offset::Absolute(
                    (self.offset.absolute(direction, bounds, content_bounds)
                        - delta)
                        .max(0.0)
                        .min((content_bounds.width - bounds.width) as f32),
                )
            }
        }
    }

    /// Scrolls the scrollable to a relative amount.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    pub fn scroll_to(
        &mut self,
        direction: &Direction,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.snap_to(percentage);
        self.unsnap(direction, bounds, content_bounds);
    }

    /// Snaps the scroll position to a relative amount.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    pub fn snap_to(&mut self, percentage: f32) {
        self.offset = Offset::Relative(percentage.max(0.0).min(1.0));
    }

    /// Unsnaps the current scroll position, if snapped, given the bounds of the
    /// scrollable and its contents.
    pub fn unsnap(
        &mut self,
        direction: &Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset = Offset::Absolute(self.offset.absolute(
            direction,
            bounds,
            content_bounds,
        ));
    }

    /// Returns the current scrolling offset of the [`State`], given the bounds
    /// of the scrollable and its contents.
    pub fn offset(
        &self,
        direction: &Direction,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) -> u32 {
        self.offset.absolute(direction, bounds, content_bounds) as u32
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

/// The scrollbar of a scrollable.
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

    fn grab_scroller(
        &self,
        direction: &Direction,
        cursor_position: Point,
    ) -> Option<f32> {
        if self.outer_bounds.contains(cursor_position) {
            Some(if self.scroller.bounds.contains(cursor_position) {
                match direction {
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
        direction: &Direction,
        grabbed_at: f32,
        cursor_position: Point,
    ) -> f32 {
        match direction {
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

fn scrollbar(
    direction: &Direction,
    state: &State,
    scrollbar_length: u16,
    scrollbar_margin: u16,
    scroller_length: u16,
    bounds: Rectangle,
    content_bounds: Rectangle,
) -> Option<Scrollbar> {
    let offset = state.offset(direction, bounds, content_bounds);

    match direction {
        Direction::Vertical => {
            if content_bounds.height > bounds.height {
                let outer_length = scrollbar_length.max(scroller_length)
                    + 2 * scrollbar_margin;

                let outer_bounds = Rectangle {
                    x: bounds.x + bounds.width - outer_length as f32,
                    y: bounds.y,
                    width: outer_length as f32,
                    height: bounds.height,
                };

                let scrollbar_bounds = Rectangle {
                    x: bounds.x + bounds.width
                        - f32::from(outer_length / 2 + scrollbar_length / 2),
                    y: bounds.y,
                    width: scrollbar_length as f32,
                    height: bounds.height,
                };

                let ratio = bounds.height / content_bounds.height;
                let scroller_height = bounds.height * ratio;
                let y_offset = offset as f32 * ratio;

                let scroller_bounds = Rectangle {
                    x: bounds.x + bounds.width
                        - f32::from(outer_length / 2 + scroller_length / 2),
                    y: scrollbar_bounds.y + y_offset,
                    width: scroller_length as f32,
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
        Direction::Horizontal => {
            if content_bounds.width > bounds.width {
                let outer_length = scrollbar_length.max(scroller_length)
                    + 2 * scrollbar_margin;

                let outer_bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y + bounds.height - outer_length as f32,
                    width: bounds.width,
                    height: outer_length as f32,
                };

                let scrollbar_bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y + bounds.height
                        - f32::from(outer_length / 2 + scrollbar_length / 2),
                    width: bounds.width,
                    height: scrollbar_length as f32,
                };

                let ratio = bounds.width / content_bounds.width;
                let scroller_width = bounds.width * ratio;
                let x_offset = offset as f32 * ratio;

                let scroller_bounds = Rectangle {
                    x: scrollbar_bounds.x + x_offset,
                    y: bounds.y + bounds.height
                        - f32::from(outer_length / 2 + scroller_length / 2),
                    width: scroller_width,
                    height: scroller_length as f32,
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
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
struct Scroller {
    /// The bounds of the [`Scroller`].
    bounds: Rectangle,
}

/// Computes the layout of a scrollable based on its [`Direction`].
pub fn layout<Renderer>(
    direction: &Direction,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    layout_content: impl FnOnce(&Renderer, &layout::Limits) -> layout::Node,
) -> layout::Node {
    let (limits, child_limits) = match direction {
        Direction::Vertical => {
            let limits =
                limits.max_height(u32::MAX).width(width).height(height);
            let child_limits = layout::Limits::new(
                Size::new(limits.min().width, 0.0),
                Size::new(limits.max().width, f32::INFINITY),
            );
            (limits, child_limits)
        }
        Direction::Horizontal => {
            let limits = limits.max_width(u32::MAX).width(width).height(height);
            let child_limits = layout::Limits::new(
                Size::new(0.0, limits.min().height),
                Size::new(f32::INFINITY, limits.max().height),
            );
            (limits, child_limits)
        }
    };

    let content = layout_content(renderer, &child_limits);
    let size = limits.resolve(content.size());

    layout::Node::with_children(size, vec![content])
}

/// Performs an operation on the scrollable.
pub fn operate<'a, Message, Renderer>(
    tree: &mut Tree,
    id: Option<&Id>,
    content: &Element<'a, Message, Renderer>,
    layout: Layout<'_>,
    operation: &mut dyn Operation<Message>,
) where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    let state = tree.state.downcast_mut::<State>();

    operation.scrollable(state, id.map(|id| &id.0));

    operation.container(None, &mut |operation| {
        content.as_widget().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            operation,
        );
    });
}

/// Processes an [`Event`] and updates the [`State`] of a scrollable accordingly.
pub fn update<Message>(
    direction: &Direction,
    state: &mut State,
    event: Event,
    layout: Layout<'_>,
    cursor_position: Point,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    scrollbar_length: u16,
    scrollbar_margin: u16,
    scroller_length: u16,
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
        direction,
        state,
        scrollbar_length,
        scrollbar_margin,
        scroller_length,
        bounds,
        content_bounds,
    );
    let is_mouse_over_scrollbar = scrollbar
        .as_ref()
        .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
        .unwrap_or(false);

    let event_status = {
        let cursor_position = match direction {
            Direction::Vertical => {
                if is_mouse_over && !is_mouse_over_scrollbar {
                    Point::new(
                        cursor_position.x,
                        cursor_position.y
                            + state.offset(direction, bounds, content_bounds)
                                as f32,
                    )
                } else {
                    // TODO: Make `cursor_position` an `Option<Point>` so we can encode
                    // cursor availability.
                    // This will probably happen naturally once we add multi-window
                    // support.
                    Point::new(cursor_position.x, -1.0)
                }
            }
            Direction::Horizontal => {
                if is_mouse_over && !is_mouse_over_scrollbar {
                    Point::new(
                        cursor_position.x
                            + state.offset(direction, bounds, content_bounds)
                                as f32,
                        cursor_position.y,
                    )
                } else {
                    Point::new(-1.0, cursor_position.y)
                }
            }
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
                //mousewheel scrolling is only supported for vertical scrollables
                if let Direction::Vertical = direction {
                    match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            // TODO: Configurable speed (?)
                            state.scroll(
                                direction,
                                y * 60.0,
                                bounds,
                                content_bounds,
                            );
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => {
                            state.scroll(direction, y, bounds, content_bounds);
                        }
                    }

                    notify_on_scroll(
                        direction,
                        state,
                        on_scroll,
                        bounds,
                        content_bounds,
                        shell,
                    );
                }

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
                            let delta = match direction {
                                Direction::Vertical => {
                                    cursor_position.y - scroll_box_touched_at.y
                                }
                                Direction::Horizontal => {
                                    cursor_position.x - scroll_box_touched_at.x
                                }
                            };

                            state.scroll(
                                direction,
                                delta,
                                bounds,
                                content_bounds,
                            );

                            state.scroll_box_touched_at = Some(cursor_position);

                            notify_on_scroll(
                                direction,
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
                        direction,
                        scrollbar.scroll_percentage(
                            direction,
                            scroller_grabbed_at,
                            cursor_position,
                        ),
                        bounds,
                        content_bounds,
                    );

                    notify_on_scroll(
                        direction,
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
                        scrollbar.grab_scroller(direction, cursor_position)
                    {
                        state.scroll_to(
                            direction,
                            scrollbar.scroll_percentage(
                                direction,
                                scroller_grabbed_at,
                                cursor_position,
                            ),
                            bounds,
                            content_bounds,
                        );

                        state.scroller_grabbed_at = Some(scroller_grabbed_at);

                        notify_on_scroll(
                            direction,
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

fn notify_on_scroll<Message>(
    direction: &Direction,
    state: &State,
    on_scroll: &Option<Box<dyn Fn(f32) -> Message + '_>>,
    bounds: Rectangle,
    content_bounds: Rectangle,
    shell: &mut Shell<'_, Message>,
) {
    match direction {
        Direction::Vertical => {
            if content_bounds.height <= bounds.height {
                return;
            }

            if let Some(on_scroll) = on_scroll {
                shell.publish(on_scroll(
                    state.offset.absolute(direction, bounds, content_bounds)
                        / (content_bounds.height - bounds.height)
                ));
            }
        }
        Direction::Horizontal => {
            if content_bounds.width <= bounds.width {
                return;
            }

            if let Some(on_scroll) = on_scroll {
                shell.publish(on_scroll(
                    state.offset.absolute(direction, bounds, content_bounds)
                        / (content_bounds.width - bounds.width)
                ));
            }
        }
    }



}

/// Draws a scrollable.
pub fn draw<Renderer>(
    direction: &Direction,
    state: &State,
    renderer: &mut Renderer,
    theme: &Renderer::Theme,
    layout: Layout<'_>,
    cursor_position: Point,
    scrollbar_length: u16,
    scrollbar_margin: u16,
    scroller_length: u16,
    style: <Renderer::Theme as StyleSheet>::Style,
    draw_content: impl FnOnce(&mut Renderer, Layout<'_>, Point, &Rectangle),
) where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    let bounds = layout.bounds();
    let content_layout = layout.children().next().unwrap();
    let content_bounds = content_layout.bounds();
    let offset = state.offset(direction, bounds, content_bounds);
    let scrollbar = scrollbar(
        direction,
        state,
        scrollbar_length,
        scrollbar_margin,
        scroller_length,
        bounds,
        content_bounds,
    );

    let is_mouse_over = bounds.contains(cursor_position);
    let is_mouse_over_scrollbar = scrollbar
        .as_ref()
        .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
        .unwrap_or(false);

    let cursor_position = match direction {
        Direction::Vertical => {
            if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(cursor_position.x, cursor_position.y + offset as f32)
            } else {
                Point::new(cursor_position.x, -1.0)
            }
        }
        Direction::Horizontal => {
            if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(cursor_position.x + offset as f32, cursor_position.y)
            } else {
                Point::new(-1.0, cursor_position.y)
            }
        }
    };

    let translated_bounds = |bounds: Rectangle| match direction {
        Direction::Vertical => Rectangle {
            y: bounds.y + offset as f32,
            ..bounds
        },
        Direction::Horizontal => Rectangle {
            x: bounds.x + offset as f32,
            ..bounds
        },
    };

    if let Some(scrollbar) = scrollbar {
        renderer.with_layer(bounds, |renderer| {
            renderer.with_translation(
                match direction {
                    Direction::Vertical => Vector::new(0.0, -(offset as f32)),
                    Direction::Horizontal => Vector::new(-(offset as f32), 0.0),
                },
                |renderer| {
                    draw_content(
                        renderer,
                        content_layout,
                        cursor_position,
                        &translated_bounds(bounds),
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
            &translated_bounds(bounds),
        );
    }
}

/// Computes the current [`mouse::Interaction`] of a [`Scrollable`].
pub fn mouse_interaction(
    direction: &Direction,
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
        direction,
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
        let offset = state.offset(direction, bounds, content_bounds);

        let cursor_position = match direction {
            Direction::Vertical => {
                if is_mouse_over && !is_mouse_over_scrollbar {
                    Point::new(
                        cursor_position.x,
                        cursor_position.y + offset as f32,
                    )
                } else {
                    Point::new(cursor_position.x, -1.0)
                }
            }
            Direction::Horizontal => {
                if is_mouse_over && !is_mouse_over_scrollbar {
                    Point::new(
                        cursor_position.x + offset as f32,
                        cursor_position.y,
                    )
                } else {
                    Point::new(-1.0, cursor_position.y)
                }
            }
        };

        content_interaction(
            content_layout,
            cursor_position,
            &match direction {
                Direction::Vertical => Rectangle {
                    y: bounds.y + offset as f32,
                    ..bounds
                },
                Direction::Horizontal => Rectangle {
                    x: bounds.x + offset as f32,
                    ..bounds
                },
            },
        )
    }
}

fn overlay<'b, Message, Renderer>(
    direction: &'b Direction,
    tree: &'b mut Tree,
    layout: Layout<'_>,
    renderer: &Renderer,
    content: &'b Element<'b, Message, Renderer>,
) -> Option<overlay::Element<'b, Message, Renderer>>
where
    Renderer: crate::Renderer,
    Renderer::Theme: StyleSheet,
{
    content
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
            let offset = tree.state.downcast_ref::<State>().offset(
                direction,
                bounds,
                content_bounds,
            );

            match direction {
                Direction::Vertical => {
                    overlay.translate(Vector::new(0.0, -(offset as f32)))
                }
                Direction::Horizontal => {
                    overlay.translate(Vector::new(-(offset as f32), 0.0))
                }
            }
        })
}
