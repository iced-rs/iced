//! Navigate an endless amount of content with a scrollbar.
use crate::{
    column,
    input::{mouse, ButtonState},
    layout, Align, Column, Element, Event, Hasher, Layout, Length, Point,
    Rectangle, Size, Widget,
};

use std::{f32, hash::Hash, u32};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message, Renderer> {
    state: &'a mut State,
    height: Length,
    max_height: u32,
    content: Column<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Scrollable<'a, Message, Renderer> {
    /// Creates a new [`Scrollable`] with the given [`State`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
    pub fn new(state: &'a mut State) -> Self {
        Scrollable {
            state,
            height: Length::Shrink,
            max_height: u32::MAX,
            content: Column::new(),
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

    /// Sets the padding of the [`Scrollable`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn padding(mut self, units: u16) -> Self {
        self.content = self.content.padding(units);
        self
    }

    /// Sets the width of the [`Scrollable`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn width(mut self, width: Length) -> Self {
        self.content = self.content.width(width);
        self
    }

    /// Sets the height of the [`Scrollable`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Scrollable`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.content = self.content.max_width(max_width);
        self
    }

    /// Sets the maximum height of the [`Scrollable`] in pixels.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the horizontal alignment of the contents of the [`Scrollable`] .
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn align_items(mut self, align_items: Align) -> Self {
        self.content = self.content.align_items(align_items);
        self
    }

    /// Adds an element to the [`Scrollable`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn push<E>(mut self, child: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer>>,
    {
        self.content = self.content.push(child);
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Scrollable<'a, Message, Renderer>
where
    Renderer: self::Renderer + column::Renderer,
{
    fn width(&self) -> Length {
        Length::Fill
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
            .width(Length::Fill)
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
        messages: &mut Vec<Message>,
        renderer: &Renderer,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

        let is_mouse_over_scrollbar = renderer.is_mouse_over_scrollbar(
            bounds,
            content_bounds,
            cursor_position,
        );

        // TODO: Event capture. Nested scrollables should capture scroll events.
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
                }
                _ => {}
            }
        }

        if self.state.is_scrollbar_grabbed() || is_mouse_over_scrollbar {
            match event {
                Event::Mouse(mouse::Event::Input {
                    button: mouse::Button::Left,
                    state,
                }) => match state {
                    ButtonState::Pressed => {
                        self.state.scroll_to(
                            cursor_position.y / (bounds.y + bounds.height),
                            bounds,
                            content_bounds,
                        );

                        self.state.scrollbar_grabbed_at = Some(cursor_position);
                    }
                    ButtonState::Released => {
                        self.state.scrollbar_grabbed_at = None;
                    }
                },
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let Some(scrollbar_grabbed_at) =
                        self.state.scrollbar_grabbed_at
                    {
                        let ratio = content_bounds.height / bounds.height;
                        let delta = scrollbar_grabbed_at.y - cursor_position.y;

                        self.state.scroll(
                            delta * ratio,
                            bounds,
                            content_bounds,
                        );

                        self.state.scrollbar_grabbed_at = Some(cursor_position);
                    }
                }
                _ => {}
            }
        }

        let cursor_position = if is_mouse_over
            && !(is_mouse_over_scrollbar
                || self.state.scrollbar_grabbed_at.is_some())
        {
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
            event,
            content,
            cursor_position,
            messages,
            renderer,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> Renderer::Output {
        let bounds = layout.bounds();
        let content_layout = layout.children().next().unwrap();
        let content_bounds = content_layout.bounds();
        let offset = self.state.offset(bounds, content_bounds);

        let is_mouse_over = bounds.contains(cursor_position);
        let is_mouse_over_scrollbar = renderer.is_mouse_over_scrollbar(
            bounds,
            content_bounds,
            cursor_position,
        );

        let content = {
            let cursor_position = if is_mouse_over && !is_mouse_over_scrollbar {
                Point::new(cursor_position.x, cursor_position.y + offset as f32)
            } else {
                Point::new(cursor_position.x, -1.0)
            };

            self.content.draw(renderer, content_layout, cursor_position)
        };

        self::Renderer::draw(
            renderer,
            &self.state,
            bounds,
            content_layout.bounds(),
            is_mouse_over,
            is_mouse_over_scrollbar,
            offset,
            content,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Scrollable<'static, (), ()>>().hash(state);

        self.height.hash(state);
        self.max_height.hash(state);

        self.content.hash_layout(state)
    }
}

/// The local state of a [`Scrollable`].
///
/// [`Scrollable`]: struct.Scrollable.html
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    scrollbar_grabbed_at: Option<Point>,
    offset: f32,
}

impl State {
    /// Creates a new [`State`] with the scrollbar located at the top.
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> Self {
        State::default()
    }

    /// Apply a scrolling offset to the current [`State`], given the bounds of
    /// the [`Scrollable`] and its contents.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
    pub fn scroll(
        &mut self,
        delta_y: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        if bounds.height >= content_bounds.height {
            return;
        }

        self.offset = (self.offset - delta_y)
            .max(0.0)
            .min((content_bounds.height - bounds.height) as f32);
    }

    /// Moves the scroll position to a relative amount, given the bounds of
    /// the [`Scrollable`] and its contents.
    ///
    /// `0` represents scrollbar at the top, while `1` represents scrollbar at
    /// the bottom.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
    pub fn scroll_to(
        &mut self,
        percentage: f32,
        bounds: Rectangle,
        content_bounds: Rectangle,
    ) {
        self.offset =
            ((content_bounds.height - bounds.height) * percentage).max(0.0);
    }

    /// Returns the current scrolling offset of the [`State`], given the bounds
    /// of the [`Scrollable`] and its contents.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
    pub fn offset(&self, bounds: Rectangle, content_bounds: Rectangle) -> u32 {
        let hidden_content =
            (content_bounds.height - bounds.height).max(0.0).round() as u32;

        self.offset.min(hidden_content as f32) as u32
    }

    /// Returns whether the scrollbar is currently grabbed or not.
    pub fn is_scrollbar_grabbed(&self) -> bool {
        self.scrollbar_grabbed_at.is_some()
    }
}

/// The renderer of a [`Scrollable`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Scrollable`] in your user interface.
///
/// [`Scrollable`]: struct.Scrollable.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer + Sized {
    /// Returns whether the mouse is over the scrollbar given the bounds of
    /// the [`Scrollable`] and its contents.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    fn is_mouse_over_scrollbar(
        &self,
        bounds: Rectangle,
        content_bounds: Rectangle,
        cursor_position: Point,
    ) -> bool;

    /// Draws the [`Scrollable`].
    ///
    /// It receives:
    /// - the [`State`] of the [`Scrollable`]
    /// - the bounds of the [`Scrollable`]
    /// - whether the mouse is over the [`Scrollable`] or not
    /// - whether the mouse is over the scrollbar or not
    /// - the scrolling offset
    /// - the drawn content
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
    fn draw(
        &mut self,
        scrollable: &State,
        bounds: Rectangle,
        content_bounds: Rectangle,
        is_mouse_over: bool,
        is_mouse_over_scrollbar: bool,
        offset: u32,
        content: Self::Output,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Scrollable<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer + column::Renderer,
    Message: 'static,
{
    fn from(
        scrollable: Scrollable<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(scrollable)
    }
}
