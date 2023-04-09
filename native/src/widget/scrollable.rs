//! Navigate an endless amount of content with a scrollbar.
use crate::column;
use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::touch;
use crate::{
    Align, Clipboard, Column, Element, Hasher, Layout, Length, Padding, Point,
    Rectangle, Size, Vector, Widget,
};

use std::{f32, hash::Hash, u32};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message, Renderer: self::Renderer> {
    state: &'a mut State,
    height: Length,
    max_height: u32,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    content: Column<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(f32) -> Message>>,
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer> Scrollable<'a, Message, Renderer> {
    /// Creates a new [`Scrollable`] with the given [`State`].
    pub fn new(state: &'a mut State) -> Self {
        Scrollable {
            state,
            height: Length::Shrink,
            max_height: u32::MAX,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            content: Column::new(),
            on_scroll: None,
            style: Renderer::Style::default(),
        }
    }

    /// Sets the vertical spacing _between_ elements.
    ///
    /// Custom margins per element do not exist in Iced. You should use this
    /// method instead! While less flexible, it helps you keep spacing between
    /// elements consistent.
    pub fn spacing(mut self, units: u16) -> Self {
        self.content = self.content.spacing(units);
        self
    }

    /// Sets the [`Padding`] of the [`Scrollable`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.content = self.content.padding(padding);
        self
    }

    /// Sets the width of the [`Scrollable`].
    pub fn width(mut self, width: Length) -> Self {
        self.content = self.content.width(width);
        self
    }

    /// Sets the height of the [`Scrollable`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Scrollable`].
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.content = self.content.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Scrollable`] in pixels.
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Scrollable`] .
    pub fn align_items(mut self, align_items: Align) -> Self {
        self.content = self.content.align_items(align_items);
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
    pub fn on_scroll(mut self, f: impl Fn(f32) -> Message + 'static) -> Self {
        self.on_scroll = Some(Box::new(f));
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

    fn notify_on_scroll(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        messages: &mut Vec<Message>,
    ) {
        if content_bounds.height <= bounds.height {
            return;
        }

        if let Some(on_scroll) = &self.on_scroll {
            messages.push(on_scroll(
                self.state.offset.absolute(bounds, content_bounds)
                    / (content_bounds.height - bounds.height),
            ));
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn width(&self) -> Length {
        Widget::<Message, Renderer>::width(&self.content)
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .max_height(self.max_height)
            .width(Widget::<Message, Renderer>::width(&self.content))
            .height(self.height);

        let child_limits = layout::Limits::new(
            Size::new(limits.min().width, 0.0),
            Size::new(limits.max().width, f32::INFINITY),
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
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
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
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
        );
        let is_mouse_over_scrollbar = scrollbar
            .as_ref()
            .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
            .unwrap_or(false);

        let event_status = {
            let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(
                    cursor_position.x,
                    cursor_position.y
                        + self.state.offset(bounds, content_bounds) as f32,
                )
            } else {
                // TODO: Make `cursor_position` an `Option<Point>` so we can encode
                // cursor availability.
                // This will probably happen naturally once we add multi-window
                // support.
                Point::new(cursor_position.x, -1.0)
            };

            self.content.on_event(
                event.clone(),
                content,
                cursor_position,
                renderer,
                clipboard,
                messages,
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
                            self.state.scroll(y * 60.0, bounds, content_bounds);
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => {
                            self.state.scroll(y, bounds, content_bounds);
                        }
                    }

                    self.notify_on_scroll(bounds, content_bounds, messages);

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
                                    cursor_position.y - scroll_box_touched_at.y;

                                self.state.scroll(
                                    delta,
                                    bounds,
                                    content_bounds,
                                );

                                self.state.scroll_box_touched_at =
                                    Some(cursor_position);

                                self.notify_on_scroll(
                                    bounds,
                                    content_bounds,
                                    messages,
                                );
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

                        self.notify_on_scroll(bounds, content_bounds, messages);

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

                            self.notify_on_scroll(
                                bounds,
                                content_bounds,
                                messages,
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
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
        );

        let is_mouse_over = bounds.contains(cursor_position);
        let is_mouse_over_scrollbar = scrollbar
            .as_ref()
            .map(|scrollbar| scrollbar.is_mouse_over(cursor_position))
            .unwrap_or(false);

        let content = {
            let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(cursor_position.x, cursor_position.y + offset as f32)
            } else {
                Point::new(cursor_position.x, -1.0)
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

        self.height.hash(state);
        self.max_height.hash(state);

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

                overlay.translate(Vector::new(0.0, -(offset as f32)))
            })
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
pub trait Renderer: column::Renderer + Sized {
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the [`Scrollbar`] given the bounds and content bounds of a
    /// [`Scrollable`].
    fn scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        offset: u32,
        scrollbar_width: u16,
        scrollbar_margin: u16,
        scroller_width: u16,
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

impl<'a, Message, Renderer> From<Scrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        scrollable: Scrollable<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(scrollable)
    }
}
