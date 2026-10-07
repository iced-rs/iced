//! A popover is a floating piece of content that appears over some element.
//!
//! Unlike a [`tooltip`], a popover is not shown on hover, and it does not
//! control its own visibility. Instead, the `popover` argument is an
//! `Option`: `Some` displays the overlay (open), and `None` hides it (closed).
//! The base is always present.
//!
//! When the user clicks outside of the popover's bounds, the popover notifies
//! the application through its `on_close` handler, which is typically how the
//! application decides to set the `popover` argument to `None` (i.e. to close
//! it).
//!
//! The popover's overlay does not capture mouse events on its own: unless
//! the popup's content handles them, events pass through the popup to the
//! layers below. If you want to capture mouse button presses inside the
//! popup's bounds, wrap the popup's content in [`opaque()`].
//!
//! [`tooltip`]: crate::tooltip::Tooltip
//! [`opaque()`]: crate::opaque()
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
//! # use iced::widget::Widget;
//! # pub struct State {
//! #     is_open: bool,
//! # }
//! use iced::widget::{button, container, popover, text};
//!
//! #[derive(Clone)]
//! enum Message {
//!     Close,
//! }
//!
//! fn view(state: &State) -> impl Widget<Message> {
//!     // The base is always present. The `popover` argument is `Some` when the
//!     // popover is open and `None` when it is closed.
//!     popover(
//!         button(text("Click me!")).on_press(Message::Close),
//!         state.is_open.then(|| {
//!             container(text("This is the popover contents!")).padding(10)
//!         }),
//!     )
//!     .position(popover::Position::Bottom)
//!     .on_close(Message::Close)
//! }
//! ```
use crate::Opaque;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::touch;
use crate::core::widget::{self, Widget};
use crate::core::{Event, Length, Pixels, Point, Rectangle, Shell, Size, Vector};

/// A floating piece of content that appears over another element.
///
/// The popover does not control its own visibility. Instead, the `popover`
/// argument is an `Option`: `Some` displays the overlay (open), and `None`
/// hides it (closed). The base is always present.
///
/// When the user clicks outside of the popover's bounds, the popover notifies
/// the application through its `on_close` handler.
///
/// The popover's overlay does not capture mouse events on its own: unless
/// the popup's content handles them, events pass through the popup to the
/// layers below. If you want to capture mouse button presses inside the
/// popup's bounds, wrap the popup's content in [`opaque()`].
///
/// [`opaque()`]: crate::opaque()
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::Renderer; pub use iced_widget::core::*; }
/// # use iced::widget::Widget;
/// # pub struct State {
/// #     is_open: bool,
/// # }
/// use iced::widget::{button, container, popover, text};
///
/// #[derive(Clone)]
/// enum Message {
///     Close,
/// }
///
/// fn view(state: &State) -> impl Widget<Message> {
///     // The base is always present. The `popover` argument is `Some` when the
///     // popover is open and `None` when it is closed.
///     popover(
///         button(text("Click me!")).on_press(Message::Close),
///         state.is_open.then(|| {
///             container(text("This is the popover contents!")).padding(10)
///         }),
///     )
///     .position(popover::Position::Bottom)
///     .on_close(Message::Close)
/// }
/// ```
pub struct Popover<W, V, Message> {
    content: W,
    popup: Option<V>,
    position: Position,
    gap: f32,
    snap_within_viewport: bool,
    on_close: Option<Message>,
}

impl<W, V, Message> Popover<W, V, Message> {
    /// Creates a new [`Popover`].
    ///
    /// It expects:
    ///   * the `content` widget that the popover is anchored to (the base), and
    ///   * the optional `popup` widget to display: `Some` when the popover is
    ///     open and `None` when it is closed.
    ///
    /// The base is always present; it is the `popup` argument that controls
    /// whether the overlay is displayed. By default, the [`Popover`] is
    /// positioned [`Position::Auto`]; use the [`Self::position`] method to set
    /// a specific position.
    pub fn new(content: W, popup: Option<V>) -> Self {
        Popover {
            content,
            popup,
            position: Position::default(),
            gap: 0.0,
            snap_within_viewport: true,
            on_close: None,
        }
    }

    /// Sets the message that will be produced when the user clicks outside of
    /// the [`Popover`]'s bounds.
    ///
    /// This is typically how the application decides to close the popover, by
    /// setting the `popover` argument to `None`.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the [`Position`] of the [`Popover`].
    ///
    /// By default, the [`Popover`] is positioned [`Position::Auto`], which
    /// places it on the side of the base with the most available space.
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Sets the gap between the content and its [`Popover`].
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets whether the [`Popover`] is snapped within the viewport.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
        self
    }
}

impl<W, V, Message> Popover<W, Opaque<V>, Message> {
    /// Makes interactions on the popup of the [`Popover`] fall through
    /// to the base layer.
    pub fn passthrough(self) -> Popover<W, V, Message> {
        Popover {
            content: self.content,
            popup: self.popup.map(Opaque::into_inner),
            position: self.position,
            gap: self.gap,
            snap_within_viewport: self.snap_within_viewport,
            on_close: self.on_close,
        }
    }
}

impl<W, V, Message> widget::Meta for Popover<W, V, Message> {}

impl<W, V, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Popover<W, V, Message>
where
    W: Widget<Message, Theme, Renderer>,
    V: Widget<Message, Theme, Renderer>,
    Message: Clone,
    Renderer: crate::core::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            needs_relayout: true,
        })
    }

    fn diff(&mut self, tree: &mut widget::Tree) {
        // The popup's contents may have changed, so the cached layout
        // (if any) is no longer valid
        tree.state.downcast_mut::<State>().needs_relayout = true;

        match self.popup.as_mut() {
            Some(popup) => {
                if tree.children.len() != 2 {
                    tree.children =
                        vec![widget::Tree::new(&self.content), widget::Tree::new(popup)];
                }

                tree.children[0].diff(&mut self.content);
                tree.children[1].diff(popup);
            }
            None => {
                if tree.children.len() != 1 {
                    tree.children = vec![widget::Tree::new(&self.content)];
                }

                tree.children[0].diff(&mut self.content);
            }
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.size()
    }

    fn layout(&mut self, tree: &mut widget::Tree, renderer: &Renderer, limits: &layout::Limits) {
        self.content.layout(&mut tree.children[0], renderer, limits);

        tree.size = tree.children[0].size;
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
        self.content.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
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
        self.content.draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds(), viewport);
        operation.traverse(&mut |operation| {
            self.content
                .operate(&mut tree.children[0], layout, viewport, renderer, operation);
        });
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
        let mut children = tree.children.iter_mut();

        let base = children.next().unwrap();
        let popup_tree = children.next();

        let mut overlays =
            self.content
                .overlay(base, layout, renderer, viewport, translation, window);

        let viewport = *viewport + translation;

        if let (Some(popup), Some(popup_tree)) = (self.popup.as_mut(), popup_tree) {
            let state = tree.state.downcast_mut::<State>();

            // (Re)compute the popup's layout if it was invalidated by
            // `Widget::diff`
            if state.needs_relayout {
                popup.layout(
                    popup_tree,
                    renderer,
                    &layout::Limits::new(
                        Size::ZERO,
                        if self.snap_within_viewport {
                            viewport.size()
                        } else {
                            Size::INFINITE
                        },
                    ),
                );

                state.needs_relayout = false;
            }

            let popup_size = popup_tree.size;

            let content_bounds = layout.bounds() + translation;

            let popup_bounds = crate::overlay::Position::from(self.position).resolve(
                content_bounds,
                popup_size,
                self.gap,
                Point::ORIGIN,
                viewport,
                self.snap_within_viewport,
            );

            overlays.push(overlay::Element::new(Box::new(Overlay {
                content: popup,
                tree: popup_tree,
                layout: Layout::new(popup_size).move_to(popup_bounds.position()),
                content_bounds,
                on_close: self.on_close.clone(),
                viewport,
                window,
            })));
        }

        overlays
    }
}

/// The position of the popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The popover will appear on the side of the widget with the most
    /// available space.
    #[default]
    Auto,
    /// The popover will appear on the top of the widget.
    Top,
    /// The popover will appear on the bottom of the widget.
    Bottom,
    /// The popover will appear on the left of the widget.
    Left,
    /// The popover will appear on the right of the widget.
    Right,
}

impl From<Position> for crate::overlay::Position {
    fn from(position: Position) -> Self {
        match position {
            Position::Auto => Self::Auto {
                preference: crate::overlay::Side::default(),
            },
            Position::Top => Self::Top,
            Position::Bottom => Self::Bottom,
            Position::Left => Self::Left,
            Position::Right => Self::Right,
        }
    }
}

struct State {
    needs_relayout: bool,
}

struct Overlay<'b, V, Message> {
    content: &'b mut V,
    tree: &'b mut widget::Tree,
    layout: Layout,
    content_bounds: Rectangle,
    on_close: Option<Message>,
    viewport: Rectangle,
    window: Size,
}

impl<V, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, V, Message>
where
    V: Widget<Message, Theme, Renderer>,
    Renderer: crate::core::Renderer,
{
    fn update(
        &mut self,
        event: &Event,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let cursor_position = cursor.position();

        let is_inside =
            cursor_position.is_some_and(|position| self.layout.bounds().contains(position));

        let is_over_base =
            cursor_position.is_some_and(|position| self.content_bounds.contains(position));

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. })
        ) && !is_inside
        {
            if is_over_base {
                // The base is responsible for its own behavior, so let the
                // event reach it.
                return;
            }

            // The user clicked outside of the popover: notify the application
            // of a close request.
            if let Some(on_close) = self.on_close.take() {
                shell.publish(on_close);
            }

            return;
        }

        self.content.update(
            self.tree,
            event,
            self.layout,
            cursor,
            renderer,
            shell,
            &self.viewport,
        );
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        cursor_position: mouse::Cursor,
    ) {
        renderer.with_layer(self.viewport, |renderer| {
            self.content.draw(
                self.tree,
                renderer,
                theme,
                style,
                self.layout,
                cursor_position,
                &self.viewport,
            );
        });
    }

    fn mouse_interaction(&self, cursor: mouse::Cursor, renderer: &Renderer) -> mouse::Interaction {
        if !cursor.is_over(self.layout.bounds()) {
            return mouse::Interaction::None;
        }

        self.content
            .mouse_interaction(self.tree, self.layout, cursor, &self.viewport, renderer)
    }

    fn operate(&mut self, renderer: &Renderer, operation: &mut dyn widget::Operation) {
        self.content
            .operate(self.tree, self.layout, &self.viewport, renderer, operation);
    }

    fn overlay<'c>(
        &'c mut self,
        renderer: &Renderer,
    ) -> Vec<overlay::Element<'c, Message, Theme, Renderer>> {
        self.content.overlay(
            self.tree,
            self.layout,
            renderer,
            &self.viewport,
            Vector::ZERO,
            self.window,
        )
    }

    /// Draws the popover on top of other overlays.
    fn index(&self) -> f32 {
        2.0
    }
}
