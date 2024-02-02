use crate::event::{self, Event};
use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::widget::tree::{self, Tree};
use crate::{
    Border, Clipboard, Color, Layout, Length, Rectangle, Shell, Size, Vector,
    Widget,
};

use std::any::Any;
use std::borrow::Borrow;

/// A generic [`Widget`].
///
/// It is useful to build composable user interfaces that do not leak
/// implementation details in their __view logic__.
///
/// If you have a [built-in widget], you should be able to use `Into<Element>`
/// to turn it into an [`Element`].
///
/// [built-in widget]: crate::widget
#[allow(missing_debug_implementations)]
pub struct Element<'a, Message, Theme, Renderer> {
    widget: Box<dyn Widget<Message, Theme, Renderer> + 'a>,
}

impl<'a, Message, Theme, Renderer> Element<'a, Message, Theme, Renderer> {
    /// Creates a new [`Element`] containing the given [`Widget`].
    pub fn new(widget: impl Widget<Message, Theme, Renderer> + 'a) -> Self
    where
        Renderer: crate::Renderer,
    {
        Self {
            widget: Box::new(widget),
        }
    }

    /// Returns a reference to the [`Widget`] of the [`Element`],
    pub fn as_widget(&self) -> &dyn Widget<Message, Theme, Renderer> {
        self.widget.as_ref()
    }

    /// Returns a mutable reference to the [`Widget`] of the [`Element`],
    pub fn as_widget_mut(
        &mut self,
    ) -> &mut dyn Widget<Message, Theme, Renderer> {
        self.widget.as_mut()
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
    /// ```no_run
    /// # mod counter {
    /// #     #[derive(Debug, Clone, Copy)]
    /// #     pub enum Message {}
    /// #     pub struct Counter;
    /// #
    /// #     impl Counter {
    /// #         pub fn view(
    /// #             &self,
    /// #         ) -> iced_core::Element<Message, (), iced_core::renderer::Null> {
    /// #             unimplemented!()
    /// #         }
    /// #     }
    /// # }
    /// #
    /// # mod iced {
    /// #     pub use iced_core::renderer::Null as Renderer;
    /// #     pub use iced_core::Element;
    /// #
    /// #     pub mod widget {
    /// #         pub struct Row<Message> {
    /// #             _t: std::marker::PhantomData<Message>,
    /// #         }
    /// #
    /// #         impl<Message> Row<Message> {
    /// #             pub fn new() -> Self {
    /// #                 unimplemented!()
    /// #             }
    /// #
    /// #             pub fn spacing(mut self, _: u32) -> Self {
    /// #                 unimplemented!()
    /// #             }
    /// #
    /// #             pub fn push(
    /// #                 mut self,
    /// #                 _: iced_core::Element<Message, (), iced_core::renderer::Null>,
    /// #             ) -> Self {
    /// #                 unimplemented!()
    /// #             }
    /// #         }
    /// #     }
    /// # }
    /// #
    /// use counter::Counter;
    ///
    /// use iced::widget::Row;
    /// use iced::{Element, Renderer};
    ///
    /// struct ManyCounters {
    ///     counters: Vec<Counter>,
    /// }
    ///
    /// #[derive(Debug, Clone, Copy)]
    /// pub enum Message {
    ///     Counter(usize, counter::Message),
    /// }
    ///
    /// impl ManyCounters {
    ///     pub fn view(&mut self) -> Row<Message> {
    ///         // We can quickly populate a `Row` by folding over our counters
    ///         self.counters.iter_mut().enumerate().fold(
    ///             Row::new().spacing(20),
    ///             |row, (index, counter)| {
    ///                 // We display the counter
    ///                 let element: Element<counter::Message, _, _> =
    ///                     counter.view().into();
    ///
    ///                 row.push(
    ///                     // Here we turn our `Element<counter::Message>` into
    ///                     // an `Element<Message>` by combining the `index` and the
    ///                     // message of the `element`.
    ///                     element
    ///                         .map(move |message| Message::Counter(index, message)),
    ///                 )
    ///             },
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
    pub fn map<B>(
        self,
        f: impl Fn(Message) -> B + 'a,
    ) -> Element<'a, B, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: crate::Renderer + 'a,
        B: 'a,
    {
        Element::new(Map::new(self.widget, f))
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
    ) -> Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: crate::Renderer + 'a,
    {
        Element {
            widget: Box::new(Explain::new(self, color.into())),
        }
    }
}

impl<'a, Message, Theme, Renderer>
    Borrow<dyn Widget<Message, Theme, Renderer> + 'a>
    for Element<'a, Message, Theme, Renderer>
{
    fn borrow(&self) -> &(dyn Widget<Message, Theme, Renderer> + 'a) {
        self.widget.borrow()
    }
}

impl<'a, Message, Theme, Renderer>
    Borrow<dyn Widget<Message, Theme, Renderer> + 'a>
    for &Element<'a, Message, Theme, Renderer>
{
    fn borrow(&self) -> &(dyn Widget<Message, Theme, Renderer> + 'a) {
        self.widget.borrow()
    }
}

struct Map<'a, A, B, Theme, Renderer> {
    widget: Box<dyn Widget<A, Theme, Renderer> + 'a>,
    mapper: Box<dyn Fn(A) -> B + 'a>,
}

impl<'a, A, B, Theme, Renderer> Map<'a, A, B, Theme, Renderer> {
    pub fn new<F>(
        widget: Box<dyn Widget<A, Theme, Renderer> + 'a>,
        mapper: F,
    ) -> Map<'a, A, B, Theme, Renderer>
    where
        F: 'a + Fn(A) -> B,
    {
        Map {
            widget,
            mapper: Box::new(mapper),
        }
    }
}

impl<'a, A, B, Theme, Renderer> Widget<B, Theme, Renderer>
    for Map<'a, A, B, Theme, Renderer>
where
    Renderer: crate::Renderer + 'a,
    A: 'a,
    B: 'a,
{
    fn tag(&self) -> tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.widget.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.widget.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.widget.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.widget.size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.widget.size_hint()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.widget.layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<B>,
    ) {
        struct MapOperation<'a, B> {
            operation: &'a mut dyn widget::Operation<B>,
        }

        impl<'a, T, B> widget::Operation<T> for MapOperation<'a, B> {
            fn container(
                &mut self,
                id: Option<&widget::Id>,
                bounds: Rectangle,
                operate_on_children: &mut dyn FnMut(
                    &mut dyn widget::Operation<T>,
                ),
            ) {
                self.operation.container(id, bounds, &mut |operation| {
                    operate_on_children(&mut MapOperation { operation });
                });
            }

            fn focusable(
                &mut self,
                state: &mut dyn widget::operation::Focusable,
                id: Option<&widget::Id>,
            ) {
                self.operation.focusable(state, id);
            }

            fn scrollable(
                &mut self,
                state: &mut dyn widget::operation::Scrollable,
                id: Option<&widget::Id>,
                bounds: Rectangle,
                translation: Vector,
            ) {
                self.operation.scrollable(state, id, bounds, translation);
            }

            fn text_input(
                &mut self,
                state: &mut dyn widget::operation::TextInput,
                id: Option<&widget::Id>,
            ) {
                self.operation.text_input(state, id);
            }

            fn custom(&mut self, state: &mut dyn Any, id: Option<&widget::Id>) {
                self.operation.custom(state, id);
            }
        }

        self.widget.operate(
            tree,
            layout,
            renderer,
            &mut MapOperation { operation },
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, B>,
        viewport: &Rectangle,
    ) -> event::Status {
        let mut local_messages = Vec::new();
        let mut local_shell = Shell::new(&mut local_messages);

        let status = self.widget.on_event(
            tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            &mut local_shell,
            viewport,
        );

        shell.merge(local_shell, &self.mapper);

        status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.widget
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.widget
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, B, Theme, Renderer>> {
        let mapper = &self.mapper;

        self.widget
            .overlay(tree, layout, renderer, translation)
            .map(move |overlay| overlay.map(mapper))
    }
}

struct Explain<'a, Message, Theme, Renderer: crate::Renderer> {
    element: Element<'a, Message, Theme, Renderer>,
    color: Color,
}

impl<'a, Message, Theme, Renderer> Explain<'a, Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    fn new(
        element: Element<'a, Message, Theme, Renderer>,
        color: Color,
    ) -> Self {
        Explain { element, color }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Explain<'a, Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.element.widget.size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.element.widget.size_hint()
    }

    fn tag(&self) -> tree::Tag {
        self.element.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.element.widget.state()
    }

    fn children(&self) -> Vec<Tree> {
        self.element.widget.children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.element.widget.diff(tree);
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.element.widget.layout(tree, renderer, limits)
    }

    fn operate(
        &self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation<Message>,
    ) {
        self.element
            .widget
            .operate(state, layout, renderer, operation);
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        self.element.widget.on_event(
            state, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        fn explain_layout<Renderer: crate::Renderer>(
            renderer: &mut Renderer,
            color: Color,
            layout: Layout<'_>,
        ) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: Border {
                        color,
                        width: 1.0,
                        ..Border::default()
                    },
                    ..renderer::Quad::default()
                },
                Color::TRANSPARENT,
            );

            for child in layout.children() {
                explain_layout(renderer, color, child);
            }
        }

        self.element
            .widget
            .draw(state, renderer, theme, style, layout, cursor, viewport);

        explain_layout(renderer, self.color, layout);
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element
            .widget
            .mouse_interaction(state, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.element
            .widget
            .overlay(state, layout, renderer, translation)
    }
}
