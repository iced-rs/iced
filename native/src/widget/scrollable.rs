//! Navigate an endless amount of content with a scrollbar.
use crate::{
    column, layout, mouse, overlay, Align, Clipboard, Column, Element, Event,
    Hasher, Layout, Length, Point, Rectangle, Size, Vector, Widget,
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
    style: Renderer::Style,
}

impl<'a, Message, Renderer: self::Renderer> Scrollable<'a, Message, Renderer> {
    /// Creates a new [`Scrollable`] with the given [`State`].
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
    pub fn new(state: &'a mut State) -> Self {
        Scrollable {
            state,
            height: Length::Shrink,
            max_height: u32::MAX,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            content: Column::new(),
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

    /// Sets the scrollbar width of the [`Scrollable`] .
    /// Silently enforces a minimum value of 1.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn scrollbar_width(mut self, scrollbar_width: u16) -> Self {
        self.scrollbar_width = scrollbar_width.max(1);
        self
    }

    /// Sets the scrollbar margin of the [`Scrollable`] .
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn scrollbar_margin(mut self, scrollbar_margin: u16) -> Self {
        self.scrollbar_margin = scrollbar_margin;
        self
    }

    /// Sets the scroller width of the [`Scrollable`] .
    /// Silently enforces a minimum value of 1.
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn scroller_width(mut self, scroller_width: u16) -> Self {
        self.scroller_width = scroller_width.max(1);
        self
    }

    /// Sets the style of the [`Scrollable`] .
    ///
    /// [`Scrollable`]: struct.Scrollable.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
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
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let content = layout.children().next().unwrap();
        let content_bounds = content.bounds();

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

        if self.state.is_scroller_grabbed() {
            match event {
                Event::Mouse(mouse::Event::ButtonReleased(
                    mouse::Button::Left,
                )) => {
                    self.state.scroller_grabbed_at = None;
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
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
                    }
                }
                _ => {}
            }
        } else if is_mouse_over_scrollbar {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(
                    mouse::Button::Left,
                )) => {
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
                        }
                    }
                }
                _ => {}
            }
        }

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
            event,
            content,
            cursor_position,
            messages,
            renderer,
            clipboard,
        )
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
///
/// [`Scrollable`]: struct.Scrollable.html
#[derive(Debug, Clone, Copy, Default)]
pub struct State {
    scroller_grabbed_at: Option<f32>,
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

    /// Returns whether the scroller is currently grabbed or not.
    pub fn is_scroller_grabbed(&self) -> bool {
        self.scroller_grabbed_at.is_some()
    }
}

/// The scrollbar of a [`Scrollable`].
///
/// [`Scrollable`]: struct.Scrollable.html
#[derive(Debug)]
pub struct Scrollbar {
    /// The outer bounds of the scrollable, including the [`Scrollbar`] and
    /// [`Scroller`].
    ///
    /// [`Scrollbar`]: struct.Scrollbar.html
    /// [`Scroller`]: struct.Scroller.html
    pub outer_bounds: Rectangle,

    /// The bounds of the [`Scrollbar`].
    ///
    /// [`Scrollbar`]: struct.Scrollbar.html
    pub bounds: Rectangle,

    /// The margin within the [`Scrollbar`].
    ///
    /// [`Scrollbar`]: struct.Scrollbar.html
    pub margin: u16,

    /// The bounds of the [`Scroller`].
    ///
    /// [`Scroller`]: struct.Scroller.html
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
///
/// [`Scrollbar`]: struct.Scrollbar.html
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    /// The bounds of the [`Scroller`].
    ///
    /// [`Scroller`]: struct.Scrollbar.html
    pub bounds: Rectangle,
}

/// The renderer of a [`Scrollable`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Scrollable`] in your user interface.
///
/// [`Scrollable`]: struct.Scrollable.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: column::Renderer + Sized {
    /// The style supported by this renderer.
    type Style: Default;

    /// Returns the [`Scrollbar`] given the bounds and content bounds of a
    /// [`Scrollable`].
    ///
    /// [`Scrollbar`]: struct.Scrollbar.html
    /// [`Scrollable`]: struct.Scrollable.html
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
    ///
    /// [`Scrollbar`]: struct.Scrollbar.html
    /// [`Scrollable`]: struct.Scrollable.html
    /// [`State`]: struct.State.html
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
