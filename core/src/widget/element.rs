use crate::layout;
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::widget;
use crate::widget::tree::{self, Tree};
use crate::{Border, Color, Event, Layout, Length, Rectangle, Shell, Size, Vector, Widget};

/// A generic [`Widget`].
///
/// It is useful to build composable user interfaces that do not leak
/// implementation details in their __view logic__.
///
/// If you have a [built-in widget], you should be able to use `Into<Element>`
/// to turn it into an [`Element`].
///
/// [built-in widget]: crate::widget
pub struct Element<'a, Message, Theme, Renderer> {
    widget: Box<dyn Widget<Message, Theme, Renderer> + 'a>,
}

impl<'a, Message, Theme, Renderer> Element<'a, Message, Theme, Renderer> {
    /// Creates a new [`Element`] containing the given [`Widget`].
    pub(super) fn new(widget: impl Widget<Message, Theme, Renderer> + 'a) -> Self
    where
        Renderer: crate::Renderer,
    {
        Self {
            widget: Box::new(widget),
        }
    }
    /// Marks the [`Element`] as _to-be-explained_.
    ///
    /// The [`Renderer`] will explain the layout of the [`Element`] graphically.
    /// This can be very useful for debugging your layout!
    ///
    /// [`Renderer`]: crate::Renderer
    pub fn explain<C: Into<Color>>(self, color: C) -> Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a,
        Renderer: crate::Renderer + 'a,
    {
        Element {
            widget: Box::new(Explain {
                element: self,
                color: color.into(),
            }),
        }
    }
}

impl<'a, Message, Theme, Renderer> widget::Node for Element<'a, Message, Theme, Renderer> {
    fn is_void(&self) -> bool {
        self.widget.is_void()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.widget.size()
    }

    fn tag(&self) -> tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.widget.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.widget.diff(tree);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.widget.layout(tree, renderer, limits);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.widget
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.widget
            .operate(tree, layout, viewport, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.widget
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
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
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        self.widget
            .overlay(tree, layout, renderer, viewport, translation, window)
    }

    fn boxed<'b>(self) -> Element<'b, Message, Theme, Renderer>
    where
        Self: Sized + 'b,
    {
        self
    }
}

pub struct Explain<'a, Message, Theme, Renderer: crate::Renderer> {
    element: Element<'a, Message, Theme, Renderer>,
    color: Color,
}

impl<Message, Theme, Renderer> widget::Node for Explain<'_, Message, Theme, Renderer> where
    Renderer: crate::Renderer
{
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Explain<'_, Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.element.widget.size()
    }

    fn tag(&self) -> tree::Tag {
        self.element.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.element.widget.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.element.widget.diff(tree);
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.element.widget.layout(tree, renderer, limits);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.element
            .widget
            .operate(tree, layout, viewport, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.element
            .widget
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        fn explain_layout<Renderer: crate::Renderer>(
            tree: &Tree,
            renderer: &mut Renderer,
            color: Color,
            layout: Layout,
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

            for (layout, tree) in layout.iter(&tree.children) {
                explain_layout(tree, renderer, color, layout);
            }
        }

        self.element
            .widget
            .draw(tree, renderer, theme, style, layout, cursor, viewport);

        renderer.with_layer(Rectangle::INFINITE, |renderer| {
            explain_layout(tree, renderer, self.color, layout);
        });
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.element
            .widget
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        self.element
            .widget
            .overlay(tree, layout, renderer, viewport, translation, window)
    }
}

impl<'a, T, Message, Theme, Renderer> From<Option<T>> for Element<'a, Message, Theme, Renderer>
where
    T: Into<Self>,
    Renderer: crate::Renderer,
{
    fn from(value: Option<T>) -> Self {
        value
            .map(T::into)
            .unwrap_or_else(|| Element::new(widget::Void))
    }
}
