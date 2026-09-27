//! Create custom widgets and operate on them.
pub mod operation;
pub mod text;
pub mod tree;

mod element;
mod id;

pub use element::Element;
pub use id::Id;
pub use operation::Operation;
pub use text::Text;
pub use tree::Tree;

use crate::layout::{self, Layout};
use crate::mouse;
use crate::overlay;
use crate::renderer;
use crate::shell;
use crate::{Event, Length, Rectangle, Shell, Size, Vector};

/// A component that displays information and allows interaction.
///
/// If you want to build your own widgets, you will need to implement this
/// trait.
///
/// # Examples
/// The repository has some [examples] showcasing how to implement a custom
/// widget:
///
/// - [`custom_widget`], a demonstration of how to build a custom widget that
///   draws a circle.
/// - [`geometry`], a custom widget showcasing how to draw geometry with the
///   `Mesh2D` primitive in [`iced_wgpu`].
///
/// [examples]: https://github.com/iced-rs/iced/tree/master/examples
/// [`custom_widget`]: https://github.com/iced-rs/iced/tree/master/examples/custom_widget
/// [`geometry`]: https://github.com/iced-rs/iced/tree/master/examples/geometry
/// [`iced_wgpu`]: https://github.com/iced-rs/iced/tree/master/wgpu
pub trait Widget<Message, Theme, Renderer>
where
    Renderer: crate::Renderer,
{
    /// Returns the [`Size`] of the [`Widget`] in lengths.
    fn size(&self) -> Size<Length>;

    /// Lays out the [`Widget`].
    ///
    /// This computes the [`Layout`] of the [`Widget`] and stores the result
    /// in the provided [`Tree`].
    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits);

    /// Draws the [`Widget`] using the associated `Renderer`.
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    );

    /// Returns the [`Tag`] of the [`Widget`].
    ///
    /// [`Tag`]: tree::Tag
    fn tag(&self) -> tree::Tag {
        tree::Tag::stateless()
    }

    /// Returns the [`State`] of the [`Widget`].
    ///
    /// [`State`]: tree::State
    fn state(&self) -> tree::State {
        tree::State::None
    }

    /// Reconciles the [`Widget`] with the provided [`Tree`].
    fn diff(&mut self, tree: &mut Tree) {
        tree.children.clear();
    }

    /// Applies an [`Operation`] to the [`Widget`].
    fn operate(
        &mut self,
        _tree: &mut Tree,
        _layout: Layout,
        _viewport: &Rectangle,
        _renderer: &Renderer,
        _operation: &mut dyn Operation,
    ) {
    }

    /// Processes a runtime [`Event`].
    ///
    /// By default, it does nothing.
    fn update(
        &mut self,
        _tree: &mut Tree,
        _event: &Event,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
    }

    /// Returns the current [`mouse::Interaction`] of the [`Widget`].
    ///
    /// By default, it returns [`mouse::Interaction::None`].
    fn mouse_interaction(
        &self,
        _tree: &Tree,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        mouse::Interaction::None
    }

    /// Returns the overlays of the [`Widget`].
    fn overlay<'a>(
        &'a mut self,
        _tree: &'a mut Tree,
        _layout: Layout,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
        _window: Size,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        Vec::new()
    }

    /// Returns whether the [`Widget`] is [`Void`].
    fn is_void(&self) -> bool {
        false
    }

    /// TODO
    fn map<F, B>(self, f: F) -> Map<Self, F, Message>
    where
        Self: Sized,
        F: Fn(Message) -> B,
    {
        Map {
            widget: self,
            mapper: f,
            _input: std::marker::PhantomData,
        }
    }

    /// TODO
    fn boxed<'a>(self) -> Element<'a, Message, Theme, Renderer>
    where
        Self: Sized + 'a,
    {
        Element::new(self)
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for &mut T
where
    Renderer: crate::Renderer,
    T: Widget<Message, Theme, Renderer>,
{
    fn size(&self) -> Size<Length> {
        T::size(self)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        T::layout(self, tree, renderer, limits);
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
        T::draw(self, tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn tag(&self) -> tree::Tag {
        T::tag(self)
    }

    fn state(&self) -> tree::State {
        T::state(self)
    }

    fn diff(&mut self, tree: &mut Tree) {
        T::diff(self, tree);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        T::operate(self, tree, layout, viewport, renderer, operation);
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
        T::update(self, tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        T::mouse_interaction(self, tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        T::overlay(self, tree, layout, renderer, viewport, translation, window)
    }

    fn is_void(&self) -> bool {
        T::is_void(self)
    }
}

/// A zero-sized [`Widget`] that does nothing and will be filtered out by containers.
pub struct Void;

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Void
where
    Renderer: crate::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &Renderer, _limits: &layout::Limits) {}

    fn draw(
        &self,
        _tree: &Tree,
        _renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
    }

    fn is_void(&self) -> bool {
        true
    }
}

/// TODO
pub struct Map<W, F, A> {
    widget: W,
    mapper: F,
    _input: std::marker::PhantomData<A>,
}

impl<W, A, B, F, Theme, Renderer> Widget<B, Theme, Renderer> for Map<W, F, A>
where
    W: Widget<A, Theme, Renderer>,
    B: 'static,
    F: Fn(A) -> B,
    Theme: 'static,
    Renderer: crate::Renderer + 'static,
{
    fn tag(&self) -> tree::Tag {
        self.widget.tag()
    }

    fn state(&self) -> tree::State {
        self.widget.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.widget.diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.widget.size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.widget.layout(tree, renderer, limits);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
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
        shell: &mut Shell<'_, B>,
        viewport: &Rectangle,
    ) {
        let mut local_messages = shell::Bus::new();
        let mut local_shell = shell.local(&mut local_messages);

        self.widget.update(
            tree,
            event,
            layout,
            cursor,
            renderer,
            &mut local_shell,
            viewport,
        );

        shell.merge(local_shell, &self.mapper);
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
    ) -> Vec<overlay::Element<'b, B, Theme, Renderer>> {
        let mapper = &self.mapper;

        self.widget
            .overlay(tree, layout, renderer, viewport, translation, window)
            .into_iter()
            .map(move |overlay| overlay.map(mapper))
            .collect()
    }
}

impl<'a, W, F, A, Message, Theme, Renderer> From<Map<W, F, A>>
    for Element<'a, Message, Theme, Renderer>
where
    F: Fn(A) -> Message + 'a,
    W: Widget<A, Theme, Renderer> + 'a,
    A: 'static,
    Message: 'static,
    Theme: 'static,
    Renderer: crate::Renderer + 'static,
{
    fn from(map: Map<W, F, A>) -> Self {
        map.boxed()
    }
}

impl<T, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Option<T>
where
    T: Widget<Message, Theme, Renderer>,
    Renderer: crate::Renderer,
{
    fn size(&self) -> Size<Length> {
        let Some(widget) = self else {
            return Size {
                width: Length::Shrink,
                height: Length::Shrink,
            };
        };

        widget.size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        let Some(widget) = self else {
            return;
        };

        widget.layout(tree, renderer, limits);
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
        let Some(widget) = self else {
            return;
        };

        widget.draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn tag(&self) -> tree::Tag {
        let Some(widget) = self else {
            return tree::Tag::stateless();
        };

        widget.tag()
    }

    fn state(&self) -> tree::State {
        let Some(widget) = self else {
            return tree::State::None;
        };

        widget.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        let Some(widget) = self else {
            tree.children.clear();
            return;
        };

        widget.diff(tree);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let Some(widget) = self else {
            return;
        };

        widget.operate(tree, layout, viewport, renderer, operation);
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
        let Some(widget) = self else {
            return;
        };

        widget.update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let Some(widget) = self else {
            return mouse::Interaction::None;
        };

        widget.mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'a, Message, Theme, Renderer>> {
        let Some(widget) = self else {
            return Vec::new();
        };

        widget.overlay(tree, layout, renderer, viewport, translation, window)
    }

    fn is_void(&self) -> bool {
        self.is_none()
    }
}
