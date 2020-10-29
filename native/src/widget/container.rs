//! Decorate content and apply alignment.
use std::hash::Hash;

use crate::{
    layout, overlay, Align, Clipboard, Element, Event, Hasher, Layout, Length,
    Point, Rectangle, Widget,
};

use std::u32;

/// An element decorating some content.
///
/// It is normally used for alignment purposes.
#[allow(missing_debug_implementations)]
pub struct Container<'a, Message, Renderer: self::Renderer> {
    padding: u16,
    width: Length,
    height: Length,
    max_width: u32,
    max_height: u32,
    horizontal_alignment: Align,
    vertical_alignment: Align,
    style: Renderer::Style,
    content: Element<'a, Message, Renderer>,
}

impl<'a, Message, Renderer> Container<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    /// Creates an empty [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        Container {
            padding: 0,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            horizontal_alignment: Align::Start,
            vertical_alignment: Align::Start,
            style: Renderer::Style::default(),
            content: content.into(),
        }
    }

    /// Sets the padding of the [`Container`].
    ///
    /// [`Container`]: struct.Column.html
    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    /// Sets the width of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the maximum width of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets the maximum height of the [`Container`] in pixels.
    ///
    /// [`Container`]: struct.Container.html
    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Sets the content alignment for the horizontal axis of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn align_x(mut self, alignment: Align) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the content alignment for the vertical axis of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn align_y(mut self, alignment: Align) -> Self {
        self.vertical_alignment = alignment;
        self
    }

    /// Centers the contents in the horizontal axis of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn center_x(mut self) -> Self {
        self.horizontal_alignment = Align::Center;
        self
    }

    /// Centers the contents in the vertical axis of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn center_y(mut self) -> Self {
        self.vertical_alignment = Align::Center;
        self
    }

    /// Sets the style of the [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    pub fn style(mut self, style: impl Into<Renderer::Style>) -> Self {
        self.style = style.into();
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Container<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
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
        let padding = f32::from(self.padding);

        let limits = limits
            .loose()
            .max_width(self.max_width)
            .max_height(self.max_height)
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut content = self.content.layout(renderer, &limits.loose());
        let size = limits.resolve(content.size());

        content.move_to(Point::new(padding, padding));
        content.align(self.horizontal_alignment, self.vertical_alignment, size);

        layout::Node::with_children(size.pad(padding), vec![content])
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
        self.content.widget.on_event(
            event,
            layout.children().next().unwrap(),
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
        viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            layout.bounds(),
            cursor_position,
            viewport,
            &self.style,
            &self.content,
            layout.children().next().unwrap(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.padding.hash(state);
        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);

        self.content.hash_layout(state);
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer>> {
        self.content.overlay(layout.children().next().unwrap())
    }
}

/// The renderer of a [`Container`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use a [`Container`] in your user interface.
///
/// [`Container`]: struct.Container.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// The style supported by this renderer.
    type Style: Default;

    /// Draws a [`Container`].
    ///
    /// [`Container`]: struct.Container.html
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        bounds: Rectangle,
        cursor_position: Point,
        viewport: &Rectangle,
        style: &Self::Style,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Container<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a,
{
    fn from(
        column: Container<'a, Message, Renderer>,
    ) -> Element<'a, Message, Renderer> {
        Element::new(column)
    }
}
