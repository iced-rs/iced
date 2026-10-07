//! Wraps the given widget and captures any mouse button presses inside the bounds of
//! the widget—effectively making it _opaque_.
//!
//! This widget is meant to be used to mark elements in a [`Stack`] to avoid mouse
//! events from passing through layers.
//!
//! [`Stack`]: crate::Stack
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # use iced::widget::Widget;
//! # pub type State = ();
//! use iced::widget::{button, opaque, stack, text};
//!
//! #[derive(Clone)]
//! enum Message {
//!     ButtonPressed,
//! }
//!
//! fn view(state: &State) -> impl Widget<Message> {
//!     stack![
//!         // This button lies below the opaque top layer: mouse events over
//!         // the top layer do not pass through to it.
//!         button("Click me!").on_press(Message::ButtonPressed),
//!         opaque(text("I am opaque")),
//!     ]
//! }
//! ```
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::tree::{self, Tree};
use crate::core::widget::{self, Operation, Widget};
use crate::core::{Element, Event, Layout, Length, Rectangle, Shell, Size, Vector};

/// Wraps the given widget and captures any mouse button presses inside the bounds of
/// the widget—effectively making it _opaque_.
///
/// This widget is meant to be used to mark elements in a [`Stack`] to avoid mouse
/// events from passing through layers.
///
/// [`Stack`]: crate::Stack
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # use iced::widget::Widget;
/// # pub type State = ();
/// use iced::widget::{button, opaque, stack, text};
///
/// #[derive(Clone)]
/// enum Message {
///     ButtonPressed,
/// }
///
/// fn view(state: &State) -> impl Widget<Message> {
///     stack![
///         // This button lies below the opaque top layer: mouse events over
///         // the top layer do not pass through to it.
///         button("Click me!").on_press(Message::ButtonPressed),
///         opaque(text("I am opaque")),
///     ]
/// }
/// ```
pub struct Opaque<W> {
    content: W,
}

impl<W> Opaque<W> {
    /// Creates a new [`Opaque`].
    pub fn new(content: W) -> Self {
        Opaque { content }
    }

    /// Returns the wrapped content.
    pub fn into_inner(self) -> W {
        self.content
    }
}

impl<W> widget::Meta for Opaque<W> {}

impl<W, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Opaque<W>
where
    W: Widget<Message, Theme, Renderer>,
{
    fn tag(&self) -> tree::Tag {
        self.content.tag()
    }

    fn state(&self) -> tree::State {
        self.content.state()
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.content.layout(&mut tree.children[0], renderer, limits);

        tree.size = tree.children[0].size;
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
        let (layout, tree) = layout.iter(&tree.children).next().unwrap();

        self.content
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
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
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        let is_mouse_press = matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_)));

        self.content
            .update(tree, event, layout, cursor, renderer, shell, viewport);

        if is_mouse_press && cursor.is_over(layout.bounds()) {
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let (layout, tree) = layout.iter(&tree.children).next().unwrap();

        let interaction = self
            .content
            .mouse_interaction(tree, layout, cursor, viewport, renderer);

        if interaction == mouse::Interaction::None && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Idle
        } else {
            interaction
        }
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
        let (layout, tree) = layout.iter_mut(&mut tree.children).next().unwrap();

        self.content
            .overlay(tree, layout, renderer, viewport, translation, window)
    }
}

impl<'a, W, Message, Theme, Renderer> From<Opaque<W>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a,
    W: Widget<Message, Theme, Renderer> + 'a,
{
    fn from(opaque: Opaque<W>) -> Element<'a, Message, Theme, Renderer> {
        opaque._boxed()
    }
}
