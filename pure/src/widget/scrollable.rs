use crate::widget::{Column, Tree};
use crate::{Element, Widget};

use iced_native::event::{self, Event};
use iced_native::layout::{self, Layout};
use iced_native::mouse;
use iced_native::renderer;
use iced_native::widget::scrollable;
use iced_native::{
    Alignment, Clipboard, Hasher, Length, Padding, Point, Rectangle, Shell,
};

pub use iced_style::scrollable::StyleSheet;

use std::any::{self, Any};

/// A widget that can vertically display an infinite amount of content with a
/// scrollbar.
#[allow(missing_debug_implementations)]
pub struct Scrollable<'a, Message, Renderer> {
    height: Length,
    scrollbar_width: u16,
    scrollbar_margin: u16,
    scroller_width: u16,
    content: Column<'a, Message, Renderer>,
    on_scroll: Option<Box<dyn Fn(f32) -> Message>>,
    style_sheet: Box<dyn StyleSheet + 'a>,
}

impl<'a, Message, Renderer: iced_native::Renderer>
    Scrollable<'a, Message, Renderer>
{
    /// Creates a new [`Scrollable`] with the given [`State`].
    pub fn new() -> Self {
        Scrollable {
            height: Length::Shrink,
            scrollbar_width: 10,
            scrollbar_margin: 0,
            scroller_width: 10,
            content: Column::new(),
            on_scroll: None,
            style_sheet: Default::default(),
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

    /// Sets the horizontal alignment of the contents of the [`Scrollable`] .
    pub fn align_items(mut self, align_items: Alignment) -> Self {
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
    pub fn style(
        mut self,
        style_sheet: impl Into<Box<dyn StyleSheet + 'a>>,
    ) -> Self {
        self.style_sheet = style_sheet.into();
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
    for Scrollable<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
{
    fn tag(&self) -> any::TypeId {
        any::TypeId::of::<scrollable::State>()
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(scrollable::State::new())
    }

    fn children(&self) -> &[Element<Message, Renderer>] {
        self.content.children()
    }

    fn width(&self) -> Length {
        Widget::<Message, Renderer>::width(&self.content)
    }

    fn height(&self) -> Length {
        self.height
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        self.tag().hash(state);
        self.height.hash(state);
        self.content.hash_layout(state)
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        scrollable::layout(
            renderer,
            limits,
            Widget::<Message, Renderer>::width(self),
            self.height,
            |renderer, limits| self.content.layout(renderer, limits),
        )
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
        scrollable::update(
            tree.state.downcast_mut::<scrollable::State>(),
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
                self.content.on_event(
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
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        scrollable::draw(
            tree.state.downcast_ref::<scrollable::State>(),
            renderer,
            layout,
            cursor_position,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
            self.style_sheet.as_ref(),
            |renderer, layout, cursor_position, viewport| {
                self.content.draw(
                    &tree.children[0],
                    renderer,
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
        scrollable::mouse_interaction(
            tree.state.downcast_ref::<scrollable::State>(),
            layout,
            cursor_position,
            self.scrollbar_width,
            self.scrollbar_margin,
            self.scroller_width,
            |layout, cursor_position, viewport| {
                self.content.mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor_position,
                    viewport,
                    renderer,
                )
            },
        )
    }
}
