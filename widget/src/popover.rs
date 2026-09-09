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
//! By default, the base is opaque: mouse button presses inside its bounds are
//! captured, and mouse events do not pass through it to the layers below. Use
//! [`Popover::passthrough`] to make the base transparent.
//!
//! [`tooltip`]: crate::tooltip::Tooltip
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{button, container, popover, text};
//!
//! #[derive(Clone)]
//! enum Message {
//!     Close,
//! }
//!
//! struct State {
//!     is_open: bool,
//! }
//!
//! fn view(state: &State) -> Element<'static, Message> {
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
//!     .into()
//! }
//! ```
use crate::Opaque;
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget::{self, Widget};
use crate::core::{Element, Event, Length, Pixels, Point, Rectangle, Shell, Size, Vector};

/// A floating piece of content that appears over another element.
///
/// The popover does not control its own visibility. Instead, the `popover`
/// argument is an `Option`: `Some` displays the overlay (open), and `None`
/// hides it (closed). The base is always present.
///
/// When the user clicks outside of the popover's bounds, the popover notifies
/// the application through its `on_close` handler.
///
/// By default, the base is opaque: mouse button presses inside its bounds are
/// captured, and mouse events do not pass through it to the layers below. Use
/// [`Popover::passthrough`] to make the base transparent.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{button, container, popover, text};
///
/// #[derive(Clone)]
/// enum Message {
///     Close,
/// }
///
/// struct State {
///     is_open: bool,
/// }
///
/// fn view(state: &State) -> Element<'static, Message> {
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
///     .into()
/// }
/// ```
pub struct Popover<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
{
    content: Content<'a, Message, Theme, Renderer>,
    popover: Option<Element<'a, Message, Theme, Renderer>>,
    position: Position,
    gap: f32,
    snap_within_viewport: bool,
    on_close: Option<Message>,
}

enum Content<'a, Message, Theme, Renderer> {
    Opaque(Opaque<'a, Message, Theme, Renderer>),
    Transparent(Element<'a, Message, Theme, Renderer>),
}

impl<'a, Message, Theme, Renderer> Content<'a, Message, Theme, Renderer>
where
    Renderer: crate::core::Renderer,
{
    fn as_widget(&self) -> &dyn Widget<Message, Theme, Renderer> {
        match self {
            Content::Opaque(opaque) => opaque,
            Content::Transparent(element) => element.as_widget(),
        }
    }

    fn as_widget_mut(&mut self) -> &mut dyn Widget<Message, Theme, Renderer> {
        match self {
            Content::Opaque(opaque) => opaque,
            Content::Transparent(element) => element.as_widget_mut(),
        }
    }
}

impl<'a, Message, Theme, Renderer> Popover<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    /// Creates a new [`Popover`].
    ///
    /// It expects:
    ///   * the `content` element that the popover is anchored to (the base), and
    ///   * the optional `popover` element to display: `Some` when the popover
    ///     is open and `None` when it is closed.
    ///
    /// The base is always present; it is the `popover` argument that controls
    /// whether the overlay is displayed. By default, the [`Popover`] is
    /// positioned [`Position::Auto`]; use the [`Self::position`] method to set
    /// a specific position.
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        popover: Option<impl Into<Element<'a, Message, Theme, Renderer>>>,
    ) -> Self {
        Popover {
            content: Content::Opaque(Opaque::new(content)),
            popover: popover.map(Into::into),
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

    /// Makes the base transparent, so mouse events pass through it.
    ///
    /// By default, the base is opaque: mouse button presses inside its bounds
    /// are captured, and mouse events do not pass through it to the layers
    /// below.
    pub fn passthrough(mut self) -> Self {
        if let Content::Opaque(opaque) = self.content {
            self.content = Content::Transparent(opaque.into_inner());
        }

        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Popover<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: text::Renderer,
{
    fn diff(&mut self, tree: &mut widget::Tree) {
        match self.popover.as_mut() {
            Some(popover) => {
                tree.diff_children(&mut [self.content.as_widget_mut(), popover.as_widget_mut()]);
            }
            None => {
                tree.diff_children(&mut [self.content.as_widget_mut()]);
            }
        }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
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
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        // The popover only has an overlay when it is open (i.e. when the
        // `popover` argument is `Some`).
        let popover = self.popover.as_mut()?;

        let (base, rest) = tree.children.split_at_mut(1);
        let tree = rest.first_mut()?;

        let content = self.content.as_widget_mut().overlay(
            &mut base[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let overlay = overlay::Element::new(Box::new(Overlay {
            popover,
            tree,
            content_bounds: layout.bounds() + translation,
            snap_within_viewport: self.snap_within_viewport,
            positioning: self.position,
            gap: self.gap,
            on_close: self.on_close.clone(),
            viewport: *viewport,
        }));

        let children = if let Some(content) = content {
            vec![content, overlay]
        } else {
            vec![overlay]
        };

        Some(overlay::Group::with_children(children).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Popover<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
    Theme: 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        popover: Popover<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(popover)
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

struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    popover: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut widget::Tree,
    content_bounds: Rectangle,
    snap_within_viewport: bool,
    positioning: Position,
    gap: f32,
    on_close: Option<Message>,
    viewport: Rectangle,
}

impl<Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let viewport = Rectangle::with_size(bounds);

        let layout = self.popover.as_widget_mut().layout(
            self.tree,
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

        let bounds = crate::overlay::Position::from(self.positioning).resolve(
            self.content_bounds,
            layout.size(),
            self.gap,
            Point::ORIGIN,
            viewport,
            self.snap_within_viewport,
        );

        layout.translate(Vector::new(bounds.x, bounds.y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let cursor_position = cursor.position();

        let is_inside = cursor_position.is_some_and(|position| layout.bounds().contains(position));

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

        self.popover.as_widget_mut().update(
            self.tree,
            event,
            layout,
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
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        self.popover.as_widget().draw(
            self.tree,
            renderer,
            theme,
            inherited_style,
            layout,
            cursor_position,
            &Rectangle::with_size(Size::INFINITE),
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }

        self.popover.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        )
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.popover
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.popover.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &self.viewport,
            Vector::ZERO,
        )
    }

    /// Draws the popover on top of other overlays.
    fn index(&self) -> f32 {
        2.0
    }
}
