//! Navigate an endless amount of content with a scrollbar.
use crate::row;
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::touch;
use crate::{
    Align, Clipboard, Row, Element, Hasher, Layout, Length, Point,
    Rectangle, Size, Vector, Widget,
};

use std::{f32, hash::Hash, u32};

/// A widget that can horizontally display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct ScrollableHor<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    width: Length,
    max_width: u32,
    scrollbar_height: u16,
    scrollbar_margin: u16,
    scroller_height: u16,
    content: Row<'a, Message, Renderer>,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer> ScrollableHor<'a, Message, Renderer> {
    /// Creates a new [`Scrollable`] with the given [`State`].
    pub fn new(state: &'a mut State) -> Self {
        ScrollableHor {
            state,
            width: Length::Shrink,
            max_width: u32::MAX,
            scrollbar_height: 10,
            scrollbar_margin: 0,
            scroller_height: 10,
            content: Row::new(),
            style: Renderer::Style::default(),
        }
    }

    /// Sets the horizontal spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.content = self.content.spacing(units);
        self
    }

    /// Sets the padding of the [`Scrollable`].
    pub fn padding(mut self, units: u16) -> Self {
        self.content = self.content.padding(units);
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: Length) -> Self {
        self.content = self.content.height(height);
        self
    }

    /// Sets the maximum width of the [`Scrollable`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Scrollable`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.content = self.content.max_height(max_height);
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Scrollable`] .
    pub fn align_items(mut self, align_items: Align) -> Self {
        self.content = self.content.align_items(align_items);
        self
    }

    /// Sets the scrollbar width of the [`Scrollable`] .
    /// Silently enforces a minimum value of 1.
    pub fn scrollbar_height(mut self, scrollbar_height: u16) -> Self {
        self.scrollbar_height = scrollbar_height.max(1);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollable`] .
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller width of the [`Scrollable`] .
    /// Silently enforces a minimum value of 1.
    pub fn scroller_height(mut self, scroller_height: u16) -> Self {
        self.scroller_height = scroller_height.max(1);
        self
    }

    /// Sets the style of the [`Scrollable`] .
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Adds an element to the [`Scrollable`].
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        self.content = self.content.push(child);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for ScrollableHor<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Widget::<Message, Renderer>::height(&self.content)
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .max_width(self.max_width)
            .height(Widget::<Message, Renderer>::height(&self.content))
            .width(self.width);

        let child_limits = layout::Limits::new(
            Size::new(0.0, limits.min().height),
            Size::new(f32::INFINITY, limits.max().height),
        );

        let content = self.content.layout(renderer, &child_limits);
        let size = limits.resolve(content.size());

        layout::Node::with_children(size, vec![content])
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
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        let offset = self.state.offset(bounds, content_bounds);
        let scrollbar = renderer.scrollbar(
            bounds,
            content_bounds,
            offset,
            self.scrollbar_height,
            self.scrollbar_margin,
            self.scroller_height,
        );
        let is_mouse_over_scrollbar = scrollbar
            .as_ref()
            .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
            .unwrap_or(false);

        let event_status = {
            let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(
                    cursor_position.x+ self.state.offset(bounds, content_bounds) as f32,
                    cursor_position.y,
                )
            } else {
                // TODO: Make `cursor_position` an `Option<Point>` so we can encode
                // cursor availability.
                // This will probably happen naturally once we add multi-window
                // support.
                Point::new(-1.0, cursor_position.y)
            };

            self.content.on_event(
                event.clone(),
                content,
                cursor_position,
                messages,
                renderer,
                clipboard,
            )
        };

        if let event::Status::Captured = event_status {
            return event::Status::Captured;
        }

        if is_mouse_over {
            match event {
                Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    match delta {
                        mouse::ScrollDelta::Lines { x, .. } => {
                            // TODO: Configurable speed (?)
                            self.state.scroll(x * 60.0, bounds, content_bounds);
                        }
                        mouse::ScrollDelta::Pixels { x, .. } => {
                            self.state.scroll(x, bounds, content_bounds);
                        }
                    }

                    return event::Status::Captured;
                }
                Event::Touch(event) => {
                    match event {
                        touch::Event::FingerPressed { .. } => {
                            self.state.scroll_box_touched_at =
                                Some(cursor_position);
                        }
                        touch::Event::FingerMoved { .. } => {
                            if let Some(scroll_box_touched_at) =
                                self.state.scroll_box_touched_at
                            {
                                let delta =
                                    cursor_position.x - scroll_box_touched_at.x;

                                self.state.scroll(
                                    delta,
                                    bounds,
                                    content_bounds,
                                );

                                self.state.scroll_box_touched_at =
                                    Some(cursor_position);
                            }
                        }
                        touch::Event::FingerLifted { .. }
                        | touch::Event::FingerLost { .. } => {
                            self.state.scroll_box_touched_at = None;
                        }
                    }

                    return event::Status::Captured;
                }
                _ => {}
            }
        }

        if self.state.is_scroller_grabbed() {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    self.state.scroller_grabbed_at = None;

                    return event::Status::Captured;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    if let (Some(scrollbar), Some(scroller_grabbed_at)) =
                        (scrollbar, self.state.scroller_grabbed_at)
                    {
                        self.state.scroll_to(
                            scrollbar.scroll_percentage(
                                scroller_grabbed_at,
                                cursor_position,
                            ),
                            bounds,
                            content_bounds,
                        );

                        return event::Status::Captured;
                    }
                }
                _ => {}
            }
        } else if is_mouse_over_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                ))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(scrollbar) = scrollbar {
                        if let Some(scroller_grabbed_at) =
                            scrollbar.grab_scroller(cursor_position)
                        {
                            self.state.scroll_to(
                                scrollbar.scroll_percentage(
                                    scroller_grabbed_at,
                                    cursor_position,
                                ),
                                bounds,
                                content_bounds,
                            );

                            self.state.scroller_grabbed_at =
                                Some(scroller_grabbed_at);

                            return event::Status::Captured;
                        }
                    }
                }
                _ => {}
            }
        }

        event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let offset = self.state.offset(bounds, content_bounds);
        let scrollbar = renderer.scrollbar(
            bounds,
            content_bounds,
            offset,
            self.scrollbar_height,
            self.scrollbar_margin,
            self.scroller_height,
        );

        let is_mouse_over = bounds.contains(cursor_position);
        let is_mouse_over_scrollbar = scrollbar
            .as_ref()
            .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
            .unwrap_or(false);

        let content = {
            let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(cursor_position.x  + offset as f32, cursor_position.y)
            } else {
                Point::new(-1.0, cursor_position.y)
            };

            self.content.draw(
                renderer,
                defaults,
                content_layout,
                cursor_position,
                &Rectangle {
                    y: bounds.y + offset as f32,
                    ..bounds
                },
            )
        };

        self::Renderer::draw(
            renderer,
            &self.state,
            bounds,
            content_layout.bounds(),
            is_mouse_over,
            is_mouse_over_scrollbar,
            scrollbar,
            offset,
            &self.style,
            content,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.max_width.hash(state);

        self.content.hash_layout(state)
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        let Self { content, state, .. } = self;

        content
            .overlay(layout.children().next().unwrap())
            .map(|overlay| {
                let bounds = layout.bounds();
                let content_layout = layout.children().next().unwrap();
                let content_bounds = content_layout.bounds();
                let offset = state.offset(bounds, content_bounds);

                overlay.translate(Vector::new(-(offset as f32), 0.0))
            })
    }
}

/// The local state of a [`Scrollable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    scroller_grabbed_at: Option<f32>,
    scroll_box_touched_at: Option<Point>,
    offset: f32,
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
        delta_x: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        if bounds.width >= content_bounds.width {
            return;
        }

        self.offset = (self.offset - delta_x)
            .max(0.0)
            .min((content_bounds.width - bounds.width) as f32);
    }

    /// Moves the scroll position to a relative amount, given the bounds of
    /// the [`Scrollable`] and its contents.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    pub fn scroll_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset =
            ((content_bounds.width - bounds.width) * percentage).max(0.0);
    }

    /// Returns the current scrolling offset of the [`State`], given the bounds
    /// of the [`Scrollable`] and its contents.
    pub fn offset(&self, bounds: Rectangle, content_bounds: Rectangle) -> u32 {
        let hidden_content =
            (content_bounds.width - bounds.width).max(0.0).round() as u32;

        self.offset.min(hidden_content as f32) as u32
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
pub struct Scrollbar {
    /// The outer bounds of the scrollable, including the [`Scrollbar`] and
    /// [`Scroller`].
    pub outer_bounds: Rectangle,

    /// The bounds of the [`Scrollbar`].
    pub bounds: Rectangle,

    /// The margin within the [`Scrollbar`].
    pub margin: u16,

    /// The bounds of the [`Scroller`].
    pub scroller: Scroller,
}

impl Scrollbar {
    fn is_mouse_over(&self, cursor_position: Point) -> bool {
        self.outer_bounds.contains(cursor_position)
    }

    fn grab_scroller(&self, cursor_position: Point) -> Option<f32> {
        if self.outer_bounds.contains(cursor_position) {
            Some(if self.scroller.bounds.contains(cursor_position) {
                (cursor_position.x - self.scroller.bounds.x)
                    / self.scroller.bounds.width
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
        (cursor_position.x
            - self.bounds.x
            - self.scroller.bounds.width * grabbed_at)
            / (self.bounds.width - self.scroller.bounds.width)
    }
}

/// The handle of a [`Scrollbar`].
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    /// The bounds of the [`Scroller`].
    pub bounds: Rectangle,
}

/// The renderer of a [`Scrollable`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Scrollable`] in your user interface.
///
/// [renderer]: crate::renderer
pub trait Renderer: row::Renderer + Sized {
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the [`Scrollbar`] given the bounds and content bounds of a
    /// [`Scrollable`].
    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_height: u16,
        scrollbar_height: u16,
        scroller_height: u16,
    ) -> Option<Scrollbar>;

    /// Draws the [`Scrollable`].
    ///
    /// It receives:
    /// - the [`State`] of the [`Scrollable`]
    /// - the bounds of the [`Scrollable`] widget
    /// - the bounds of the [`Scrollable`] content
    /// - whether the mouse is over the [`Scrollable`] or not
    /// - whether the mouse is over the [`Scrollbar`] or not
    /// - a optional [`Scrollbar`] to be rendered
    /// - the scrolling offset
    /// - the drawn content
    fn draw(
        &mut self,
        scrollable: &State,
        bounds: Rectangle,
        content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        scrollbar: Option<Scrollbar>,
        offset: u32,
        style: &Self::Style,
        content: Self::Output,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<ScrollableHor<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        scrollable: ScrollableHor<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(scrollable)
    }
}
