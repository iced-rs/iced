use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::{
    Clipboard, Color, Layout, Length, Point, Rectangle, Shell, Widget,
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
#[allow(missing_debug_implementations)]
pub struct Element<'a, Message, Renderer, Styling> {
    pub(crate) widget: Box<dyn Widget<Message, Renderer, Styling> + 'a>,
}

impl<'a, Message, Renderer, Styling> Element<'a, Message, Renderer, Styling>
where
    Styling: iced_style::Styling,
    Renderer: crate::Renderer<Styling>,
{
    /// Creates a new [`Element`] containing the given [`Widget`].
    pub fn new(
        widget: impl Widget<Message, Renderer, Styling> + 'a,
    ) -> Element<'a, Message, Renderer, Styling> {
        Element {
            widget: Box::new(widget),
        }
    }

    /// Applies a transformation to the produced message of the [`Element`].
    ///
    /// This method is useful when you want to decouple different parts of your
    /// UI and make them __composable__.
    ///
    /// # Example
    /// Imagine we want to use [our counter](index.html#usage). But instead of
    /// showing a single counter, we want to display many of them. We can reuse
    /// the `Counter` type as it is!
    ///
    /// We use composition to model the __state__ of our new application:
    ///
    /// ```
    /// # mod counter {
    /// #     pub struct Counter;
    /// # }
    /// use counter::Counter;
    ///
    /// struct ManyCounters {
    ///     counters: Vec<Counter>,
    /// }
    /// ```
    ///
    /// We can store the state of multiple counters now. However, the
    /// __messages__ we implemented before describe the user interactions
    /// of a __single__ counter. Right now, we need to also identify which
    /// counter is receiving user interactions. Can we use composition again?
    /// Yes.
    ///
    /// ```
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// # }
    /// #[derive(Debug, Clone, Copy)]
    /// pub enum Message {
    ///     Counter(usize, counter::Message)
    /// }
    /// ```
    ///
    /// We compose the previous __messages__ with the index of the counter
    /// producing them. Let's implement our __view logic__ now:
    ///
    /// ```
    /// # mod counter {
    /// #     type Text = iced_native::widget::Text<iced_native::renderer::Null>;
    /// #
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     impl Counter {
    /// #         pub fn view(&mut self) -> Text {
    /// #             Text::new("")
    /// #         }
    /// #     }
    /// # }
    /// #
    /// # mod iced_wgpu {
    /// #     pub use iced_native::renderer::Null as Renderer;
    /// # }
    /// #
    /// # use counter::Counter;
    /// #
    /// # struct ManyCounters {
    /// #     counters: Vec<Counter>,
    /// # }
    /// #
    /// # #[derive(Debug, Clone, Copy)]
    /// # pub enum Message {
    /// #    Counter(usize, counter::Message)
    /// # }
    /// use iced_native::Element;
    /// use iced_native::widget::Row;
    /// use iced_wgpu::Renderer;
    ///
    /// impl ManyCounters {
    ///     pub fn view(&mut self) -> Row<Message, Renderer, Styling> {
    ///         // We can quickly populate a `Row` by folding over our counters
    ///         self.counters.iter_mut().enumerate().fold(
    ///             Row::new().spacing(20),
    ///             |row, (index, counter)| {
    ///                 // We display the counter
    ///                 let element: Element<counter::Message, Renderer, Styling> =
    ///                     counter.view().into();
    ///
    ///                 row.push(
    ///                     // Here we turn our `Element<counter::Message>` into
    ///                     // an `Element<Message>` by combining the `index` and the
    ///                     // message of the `element`.
    ///                     element.map(move |message| Message::Counter(index, message))
    ///                 )
    ///             }
    ///         )
    ///     }
    /// }
    /// ```
    ///
    /// Finally, our __update logic__ is pretty straightforward: simple
    /// delegation.
    ///
    /// ```
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     impl Counter {
    /// #         pub fn update(&mut self, _message: Message) {}
    /// #     }
    /// # }
    /// #
    /// # use counter::Counter;
    /// #
    /// # struct ManyCounters {
    /// #     counters: Vec<Counter>,
    /// # }
    /// #
    /// # #[derive(Debug, Clone, Copy)]
    /// # pub enum Message {
    /// #    Counter(usize, counter::Message)
    /// # }
    /// impl ManyCounters {
    ///     pub fn update(&mut self, message: Message) {
    ///         match message {
    ///             Message::Counter(index, counter_msg) => {
    ///                 if let Some(counter) = self.counters.get_mut(index) {
    ///                     counter.update(counter_msg);
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn map<F, B>(self, f: F) -> Element<'a, B, Renderer, Styling>
    where
        Message: 'static,
        Renderer: 'a,
        Styling: 'a,
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
    /// [`Renderer`]: crate::Renderer
    pub fn explain<C: Into<Color>>(
        self,
        color: C,
    ) -> Element<'a, Message, Renderer, Styling>
    where
        Message: 'static,
        Renderer: 'a,
        Styling: 'a,
    {
        Element {
            widget: Box::new(Explain::new(self, color.into())),
        }
    }

    /// Returns the width of the [`Element`].
    pub fn width(&self) -> Length {
        self.widget.width()
    }

    /// Returns the height of the [`Element`].
    pub fn height(&self) -> Length {
        self.widget.height()
    }

    /// Computes the layout of the [`Element`] in the given [`Limits`].
    ///
    /// [`Limits`]: layout::Limits
    pub fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.widget.layout(renderer, limits)
    }

    /// Processes a runtime [`Event`].
    pub fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.widget.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    /// Draws the [`Element`] and its children using the given [`Layout`].
    pub fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Styling::Theme,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.widget
            .draw(renderer, theme, layout, cursor_position, viewport)
    }

    /// Returns the current [`mouse::Interaction`] of the [`Element`].
    pub fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.widget.mouse_interaction(
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    /// Returns the overlay of the [`Element`], if there is any.
    pub fn overlay<'b>(
        &'b mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Renderer, Styling>> {
        self.widget.overlay(layout, renderer)
    }
}

struct Map<'a, A, B, Renderer, Styling> {
    widget: Box<dyn Widget<A, Renderer, Styling> + 'a>,
    mapper: Box<dyn Fn(A) -> B>,
}

impl<'a, A, B, Renderer, Styling> Map<'a, A, B, Renderer, Styling> {
    pub fn new<F>(
        widget: Box<dyn Widget<A, Renderer, Styling> + 'a>,
        mapper: F,
    ) -> Map<'a, A, B, Renderer, Styling>
    where
        F: 'static + Fn(A) -> B,
    {
        Map {
            widget,
            mapper: Box::new(mapper),
        }
    }
}

impl<'a, A, B, Renderer, Styling> Widget<B, Renderer, Styling>
    for Map<'a, A, B, Renderer, Styling>
where
    Styling: iced_style::Styling,
    Renderer: crate::Renderer<Styling> + 'a,
    A: 'static,
    B: 'static,
{
    fn width(&self) -> Length {
        self.widget.width()
    }

    fn height(&self) -> Length {
        self.widget.height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.widget.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let status = self.widget.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            &mut local_shell,
        );

        shell.merge(local_shell, &self.mapper);

        status
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Styling::Theme,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        self.widget
            .draw(renderer, theme, layout, cursor_position, viewport)
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.widget.mouse_interaction(
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, B, Renderer, Styling>> {
        let mapper = &self.mapper;

        self.widget
            .overlay(layout, renderer)
            .map(move |overlay| overlay.map(mapper))
    }
}

struct Explain<'a, Message, Renderer, Styling> {
    element: Element<'a, Message, Renderer, Styling>,
    color: Color,
}

impl<'a, Message, Renderer, Styling> Explain<'a, Message, Renderer, Styling>
where
    Styling: iced_style::Styling,
    Renderer: crate::Renderer<Styling>,
{
    fn new(
        element: Element<'a, Message, Renderer, Styling>,
        color: Color,
    ) -> Self {
        Explain { element, color }
    }
}

impl<'a, Message, Renderer, Styling> Widget<Message, Renderer, Styling>
    for Explain<'a, Message, Renderer, Styling>
where
    Styling: iced_style::Styling,
    Renderer: crate::Renderer<Styling>,
{
    fn width(&self) -> Length {
        self.element.widget.width()
    }

    fn height(&self) -> Length {
        self.element.widget.height()
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.element.widget.layout(renderer, limits)
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.element.widget.on_event(
            event,
            layout,
            cursor_position,
            renderer,
            clipboard,
            shell,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Styling::Theme,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) {
        fn explain_layout<T, Renderer: crate::Renderer<T>>(
            renderer: &mut Renderer,
            color: Color,
            layout: Layout<'_>,
        ) where
            T: iced_style::Styling,
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border_color: color,
                    border_width: 1.0,
                    border_radius: 0.0,
                },
                Color::TRANSPARENT,
            );

            for child in layout.children() {
                explain_layout(renderer, color, child);
            }
        }

        self.element.widget.draw(
            renderer,
            theme,
            layout,
            cursor_position,
            viewport,
        );

        explain_layout(renderer, self.color, layout);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element.widget.mouse_interaction(
            layout,
            cursor_position,
            viewport,
            renderer,
        )
    }

    fn overlay(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'_, Message, Renderer, Styling>> {
        self.element.overlay(layout, renderer)
    }
}
