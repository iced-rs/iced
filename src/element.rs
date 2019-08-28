use stretch::{geometry, result};

use crate::{
    renderer, Event, Hasher, Layout, MouseCursor, Node, Point, Widget,
};

/// A generic [`Widget`].
///
/// It is useful to build composable user interfaces that do not leak
/// implementation details in their __view logic__.
///
/// If you have a [built-in widget], you should be able to use `Into<Element>`
/// to turn it into an [`Element`].
///
/// [built-in widget]: widget/index.html#built-in-widgets
/// [`Widget`]: widget/trait.Widget.html
/// [`Element`]: struct.Element.html
pub struct Element<'a, Message, Renderer> {
    pub(crate) widget: Box<dyn Widget<Message, Renderer> + 'a>,
}

impl<'a, Message, Renderer> std::fmt::Debug for Element<'a, Message, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("widget", &self.widget)
            .finish()
    }
}

impl<'a, Message, Renderer> Element<'a, Message, Renderer> {
    /// Create a new [`Element`] containing the given [`Widget`].
    ///
    /// [`Element`]: struct.Element.html
    /// [`Widget`]: widget/trait.Widget.html
    pub fn new(
        widget: impl Widget<Message, Renderer> + 'a,
    ) -> Element<'a, Message, Renderer> {
        Element {
            widget: Box::new(widget),
        }
    }

    /// Applies a transformation to the produced message of the [`Element`].
    ///
    /// This method is useful when you want to decouple different parts of your
    /// UI.
    ///
    /// [`Element`]: struct.Element.html
    ///
    /// # Example
    /// TODO
    pub fn map<F, B>(self, f: F) -> Element<'a, B, Renderer>
    where
        Message: 'static + Copy,
        Renderer: 'a,
        B: 'static,
        F: 'static + Fn(Message) -> B,
    {
        Element {
            widget: Box::new(Map::new(self.widget, f)),
        }
    }

    /// Marks the [`Element`] as _to-be-explained_.
    ///
    /// The [`Renderer`] will explain the layout of the [`Element`] graphically.
    /// This can be very useful for debugging your layout!
    ///
    /// [`Element`]: struct.Element.html
    /// [`Renderer`]: trait.Renderer.html
    pub fn explain(
        self,
        color: Renderer::Color,
    ) -> Element<'a, Message, Renderer>
    where
        Message: 'static,
        Renderer: 'a + renderer::Debugger,
    {
        Element {
            widget: Box::new(Explain::new(self, color)),
        }
    }

    pub(crate) fn compute_layout(&self, renderer: &Renderer) -> result::Layout {
        let node = self.widget.node(renderer);

        node.0.compute_layout(geometry::Size::undefined()).unwrap()
    }

    pub(crate) fn hash(&self, state: &mut Hasher) {
        self.widget.hash(state);
    }
}

struct Map<'a, A, B, Renderer> {
    widget: Box<dyn Widget<A, Renderer> + 'a>,
    mapper: Box<dyn Fn(A) -> B>,
}

impl<'a, A, B, Renderer> std::fmt::Debug for Map<'a, A, B, Renderer> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map").field("widget", &self.widget).finish()
    }
}

impl<'a, A, B, Renderer> Map<'a, A, B, Renderer> {
    pub fn new<F>(
        widget: Box<dyn Widget<A, Renderer> + 'a>,
        mapper: F,
    ) -> Map<'a, A, B, Renderer>
    where
        F: 'static + Fn(A) -> B,
    {
        Map {
            widget,
            mapper: Box::new(mapper),
        }
    }
}

impl<'a, A, B, Renderer> Widget<B, Renderer> for Map<'a, A, B, Renderer>
where
    A: Copy,
{
    fn node(&self, renderer: &Renderer) -> Node {
        self.widget.node(renderer)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<B>,
    ) {
        let mut original_messages = Vec::new();

        self.widget.on_event(
            event,
            layout,
            cursor_position,
            &mut original_messages,
        );

        original_messages
            .iter()
            .cloned()
            .for_each(|message| messages.push((self.mapper)(message)));
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor {
        self.widget.draw(renderer, layout, cursor_position)
    }

    fn hash(&self, state: &mut Hasher) {
        self.widget.hash(state);
    }
}

struct Explain<'a, Message, Renderer: renderer::Debugger> {
    element: Element<'a, Message, Renderer>,
    color: Renderer::Color,
}

impl<'a, Message, Renderer> std::fmt::Debug for Explain<'a, Message, Renderer>
where
    Renderer: renderer::Debugger,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Explain")
            .field("element", &self.element)
            .finish()
    }
}

impl<'a, Message, Renderer> Explain<'a, Message, Renderer>
where
    Renderer: renderer::Debugger,
{
    fn new(
        element: Element<'a, Message, Renderer>,
        color: Renderer::Color,
    ) -> Self {
        Explain { element, color }
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for Explain<'a, Message, Renderer>
where
    Renderer: renderer::Debugger,
{
    fn node(&self, renderer: &Renderer) -> Node {
        self.element.widget.node(renderer)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
    ) {
        self.element
            .widget
            .on_event(event, layout, cursor_position, messages)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        cursor_position: Point,
    ) -> MouseCursor {
        renderer.explain(&layout, self.color);

        self.element.widget.draw(renderer, layout, cursor_position)
    }

    fn hash(&self, state: &mut Hasher) {
        self.element.widget.hash(state);
    }
}
