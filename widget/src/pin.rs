//! A pin widget positions a widget at some fixed coordinates inside its boundaries.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; pub use iced_widget::core::Length::Fill; }
//! # use iced::widget::Widget;
//! # pub type State = ();
//! use iced::widget::pin;
//! use iced::Fill;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> impl Widget<Message> {
//!     pin("This text is displayed at coordinates (50, 50)!")
//!         .x(50)
//!         .y(50)
//! }
//! ```
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    self, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector, Widget,
};

/// A widget that positions its contents at some fixed coordinates inside of its boundaries.
///
/// By default, a [`Pin`] widget will try to fill its parent.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; pub use iced_widget::core::Length::Fill; }
/// # use iced::widget::Widget;
/// # pub type State = ();
/// use iced::widget::pin;
/// use iced::Fill;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> impl Widget<Message> {
///     pin("This text is displayed at coordinates (50, 50)!")
///         .x(50)
///         .y(50)
/// }
/// ```
pub struct Pin<W> {
    content: W,
    width: Length,
    height: Length,
    position: Point,
}

impl<W> Pin<W> {
    /// Creates a [`Pin`] widget with the given content.
    pub fn new(content: W) -> Self {
        Self {
            content,
            width: Length::Fill,
            height: Length::Fill,
            position: Point::ORIGIN,
        }
    }

    /// Sets the width of the [`Pin`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Pin`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the position of the [`Pin`]; where the pinned widget will be displayed.
    pub fn position(mut self, position: impl Into<Point>) -> Self {
        self.position = position.into();
        self
    }

    /// Sets the X coordinate of the [`Pin`].
    pub fn x(mut self, x: impl Into<Pixels>) -> Self {
        self.position.x = x.into().0;
        self
    }

    /// Sets the Y coordinate of the [`Pin`].
    pub fn y(mut self, y: impl Into<Pixels>) -> Self {
        self.position.y = y.into().0;
        self
    }
}

impl<W> widget::Node for Pin<W> {}

impl<W, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Pin<W>
where
    W: Widget<Message, Theme, Renderer>,
    Renderer: core::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        self.content.tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.state()
    }

    fn diff(&mut self, tree: &mut widget::Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(&mut self, tree: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) {
        let limits = limits.width(self.width).height(self.height);

        let available = limits.bounds() - Size::new(self.position.x, self.position.y);

        self.content.layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, available),
        );

        tree.children[0].translation = Vector::new(self.position.x, self.position.y);
        tree.size = limits.resolve(self.width, self.height, tree.children[0].size);
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
            .operate(tree, layout, viewport, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let (layout, tree) = layout.iter(&tree.children).next().unwrap();

        self.content
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if let Some(clipped_viewport) = bounds.intersection(viewport) {
            let (layout, tree) = layout.iter(&tree.children).next().unwrap();

            self.content.draw(
                tree,
                renderer,
                theme,
                style,
                layout,
                cursor,
                &clipped_viewport,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
        window: Size,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
            .overlay(tree, layout, renderer, viewport, translation, window)
    }
}
