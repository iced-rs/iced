//! A pin widget positions a widget at some fixed coordinates inside its boundaries.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::Length::Fill; }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::pin;
//! use iced::Fill;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     pin("This text is displayed at coordinates (50, 50)!")
//!         .x(50)
//!         .y(50)
//!         .into()
//! }
//! ```
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{
    self, Element, Event, Layout, Length, Pixels, Point, Rectangle, Shell, Size, Vector, Widget,
};

/// A widget that positions its contents at some fixed coordinates inside of its boundaries.
///
/// By default, a [`Pin`] widget will try to fill its parent.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::Length::Fill; }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::pin;
/// use iced::Fill;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     pin("This text is displayed at coordinates (50, 50)!")
///         .x(50)
///         .y(50)
///         .into()
/// }
/// ```
pub struct Pin<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
    position: Point,
}

impl<'a, Message, Theme, Renderer> Pin<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    /// Creates a [`Pin`] widget with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
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

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Pin<'_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
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

        self.content.as_widget_mut().layout(
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
        let (layout, tree) = layout.children_mut(tree).next().unwrap();

        self.content
            .as_widget_mut()
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
        let (layout, tree) = layout.children_mut(tree).next().unwrap();

        self.content
            .as_widget_mut()
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
        let (layout, tree) = layout.children(tree).next().unwrap();

        self.content
            .as_widget()
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
            let (layout, tree) = layout.children(tree).next().unwrap();

            self.content.as_widget().draw(
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
        let (layout, tree) = layout.children_mut(tree).next().unwrap();

        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation, window)
    }
}

impl<'a, Message, Theme, Renderer> From<Pin<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(pin: Pin<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(pin)
    }
}
