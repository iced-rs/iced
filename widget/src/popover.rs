//! A popover is a floating piece of content that appears over some element.
//!
//! Unlike a [`tooltip`], a popover is not shown on hover, and it does not
//! control its own visibility: it always displays its overlay. It is up to the
//! application to remove the popover from the view when it is "closed". The
//! base can be any element.
//!
//! When the user clicks outside of the popover's bounds, the popover notifies
//! the application through its `on_close` handler, which is typically how the
//! application decides to remove it from the view.
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
//! fn view() -> Element<'static, Message> {
//!     // The popover always displays its overlay. The application removes it
//!     // from the view when it is "closed".
//!     popover(
//!         button(text("Click me!")).on_press(Message::Close),
//!         container(text("This is the popover contents!")).padding(10),
//!         popover::Position::Bottom,
//!     )
//!     .on_close(Message::Close)
//!     .into()
//! }
//! ```
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget::{self, Widget};
use crate::core::{
    Element, Event, Length, Padding, Pixels, Point, Rectangle, Shell, Size, Vector,
};

/// A floating piece of content that appears over another element.
///
/// The popover does not control its own visibility: it always displays its
/// overlay. It is up to the application to remove the popover from the view
/// when it is "closed". The base can be any element.
///
/// When the user clicks outside of the popover's bounds, the popover notifies
/// the application through its `on_close` handler.
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
/// fn view() -> Element<'static, Message> {
///     // The popover always displays its overlay. The application removes it
///     // from the view when it is "closed".
///     popover(
///         button(text("Click me!")).on_press(Message::Close),
///         container(text("This is the popover contents!")).padding(10),
///         popover::Position::Bottom,
///     )
///     .on_close(Message::Close)
///     .into()
/// }
/// ```
pub struct Popover<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: text::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    popover: Element<'a, Message, Theme, Renderer>,
    position: Position,
    gap: f32,
    padding: f32,
    snap_within_viewport: bool,
    on_close: Option<Message>,
}

impl<'a, Message, Theme, Renderer> Popover<'a, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    /// The default padding of a [`Popover`].
    const DEFAULT_PADDING: f32 = 5.0;

    /// Creates a new [`Popover`].
    ///
    /// It expects:
    ///   * the `content` element that the popover is anchored to (the base),
    ///   * the `popover` element to display, and
    ///   * the `position` of the popover relative to the base.
    ///
    /// The popover always displays its overlay; it is up to the application to
    /// remove it from the view when it is "closed".
    pub fn new(
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
        popover: impl Into<Element<'a, Message, Theme, Renderer>>,
        position: Position,
    ) -> Self {
        Popover {
            content: content.into(),
            popover: popover.into(),
            position,
            gap: 0.0,
            padding: Self::DEFAULT_PADDING,
            snap_within_viewport: true,
            on_close: None,
        }
    }

    /// Sets the message that will be produced when the user clicks outside of
    /// the [`Popover`]'s bounds.
    ///
    /// This is typically how the application decides to remove the popover
    /// from the view (i.e. to "close" it).
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the gap between the content and its [`Popover`].
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = gap.into().0;
        self
    }

    /// Sets the padding of the [`Popover`].
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = padding.into().0;
        self
    }

    /// Sets whether the [`Popover`] is snapped within the viewport.
    pub fn snap_within_viewport(mut self, snap: bool) -> Self {
        self.snap_within_viewport = snap;
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
        tree.diff_children(&mut [
            self.content.as_widget_mut(),
            self.popover.as_widget_mut(),
        ]);
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
        let (base, rest) = tree.children.split_at_mut(1);

        let content = self.content.as_widget_mut().overlay(
            &mut base[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let popover = overlay::Element::new(Box::new(Overlay {
            position: layout.position() + translation,
            popover: &mut self.popover,
            popover_tree: &mut rest[0],
            content_bounds: layout.bounds(),
            snap_within_viewport: self.snap_within_viewport,
            positioning: self.position,
            gap: self.gap,
            padding: self.padding,
            on_close: self.on_close.clone(),
            viewport: *viewport,
        }));

        let mut children = Vec::new();
        if let Some(content) = content {
            children.push(content);
        }
        children.push(popover);

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

/// Returns whether the given [`Event`] is a press that should dismiss the
/// popover.
fn is_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

/// The position of the popover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// The popover will appear on the top of the widget.
    Top,
    /// The popover will appear on the bottom of the widget.
    #[default]
    Bottom,
    /// The popover will appear on the left of the widget.
    Left,
    /// The popover will appear on the right of the widget.
    Right,
}

struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    position: Point,
    popover: &'b mut Element<'a, Message, Theme, Renderer>,
    popover_tree: &'b mut widget::Tree,
    content_bounds: Rectangle,
    snap_within_viewport: bool,
    positioning: Position,
    gap: f32,
    padding: f32,
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

        let popover_layout = self.popover.as_widget_mut().layout(
            self.popover_tree,
            renderer,
            &layout::Limits::new(
                Size::ZERO,
                if self.snap_within_viewport {
                    viewport.size()
                } else {
                    Size::INFINITE
                },
            )
            .shrink(Padding::new(self.padding)),
        );

        let popover_bounds = popover_layout.bounds();
        let x_center = self.position.x + (self.content_bounds.width - popover_bounds.width) / 2.0;
        let y_center = self.position.y + (self.content_bounds.height - popover_bounds.height) / 2.0;

        let mut popover_rect = {
            let offset = match self.positioning {
                Position::Top => Vector::new(
                    x_center,
                    self.position.y - popover_bounds.height - self.gap - self.padding,
                ),
                Position::Bottom => Vector::new(
                    x_center,
                    self.position.y + self.content_bounds.height + self.gap + self.padding,
                ),
                Position::Left => Vector::new(
                    self.position.x - popover_bounds.width - self.gap - self.padding,
                    y_center,
                ),
                Position::Right => Vector::new(
                    self.position.x + self.content_bounds.width + self.gap + self.padding,
                    y_center,
                ),
            };

            Rectangle {
                x: offset.x - self.padding,
                y: offset.y - self.padding,
                width: popover_bounds.width + self.padding * 2.0,
                height: popover_bounds.height + self.padding * 2.0,
            }
        };

        if self.snap_within_viewport {
            if popover_rect.x < viewport.x {
                popover_rect.x = viewport.x;
            } else if viewport.x + viewport.width < popover_rect.x + popover_rect.width {
                popover_rect.x = viewport.x + viewport.width - popover_rect.width;
            }

            if popover_rect.y < viewport.y {
                popover_rect.y = viewport.y;
            } else if viewport.y + viewport.height < popover_rect.y + popover_rect.height {
                popover_rect.y = viewport.y + viewport.height - popover_rect.height;
            }
        }

        layout::Node::with_children(
            popover_rect.size(),
            vec![popover_layout.translate(Vector::new(self.padding, self.padding))],
        )
        .translate(Vector::new(popover_rect.x, popover_rect.y))
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

        let is_inside = cursor_position
            .is_some_and(|position| layout.bounds().contains(position));

        let is_over_base = cursor_position
            .is_some_and(|position| self.content_bounds.contains(position));

        if is_press(event) && !is_inside {
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

        // Forward the event to the popover contents so interactive elements
        // inside the popover keep working.
        if is_inside || !matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            let Some(popover_layout) = layout.children().next() else {
                return;
            };

            self.popover.as_widget_mut().update(
                self.popover_tree,
                event,
                popover_layout,
                cursor,
                renderer,
                shell,
                &self.viewport,
            );
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
    ) {
        // The popover content is drawn directly; users are responsible for
        // styling it.
        self.popover.as_widget().draw(
            self.popover_tree,
            renderer,
            theme,
            inherited_style,
            layout.children().next().unwrap(),
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
            self.popover_tree,
            layout.children().next().unwrap(),
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
        self.popover.as_widget_mut().operate(
            self.popover_tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    /// Draws the popover on top of other overlays.
    fn index(&self) -> f32 {
        2.0
    }
}
