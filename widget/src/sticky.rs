//! Stick elements to the viewport!
//!
//! [`Sticky`] keeps its contents visible when they would otherwise go out of
//! view, for example when they are scrolled out of a
//! [`Scrollable`](crate::Scrollable). As soon as any of the contents' edges
//! goes out of the visible bounds, they are moved into an [`overlay`] inside
//! the visible bounds, so that they stay at the edge of the viewport.
//!
//! While they are stuck, the contents keep their original layout: they
//! simply float in a new position, inside the visible bounds.
//!
//! When the edges of the contents reach the edges of their parent, they stay
//! attached to them: they keep their original bounds, and are clipped to the
//! visible bounds instead.
//!
//! Once the parent of the contents has gone out of the visible bounds, they
//! stop floating and scroll with it.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::Length::Fill; }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::{column, container, scrollable, sticky, space};
//! use iced::Fill;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     scrollable(column![
//!         sticky(container("I always stay in view!").width(Fill).padding(10)),
//!         space().height(3000),
//!     ]).into()
//! }
//! ```
use crate::core;
use crate::core::layout;
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget;
use crate::core::{Element, Event, Layout, Length, Point, Rectangle, Shell, Size, Vector, Widget};

/// A widget that keeps its contents in view.
///
/// When the contents of a [`Sticky`] widget would go out of view (for
/// example, when they are scrolled out of a [`Scrollable`](crate::Scrollable)),
/// they are moved into an [`overlay`] inside the intersection of the parent's
/// [`layout.bounds`](Layout::bounds) and the viewport, so that they stay
/// visible at the edge of the viewport.
///
/// The contents start floating as soon as any of their edges goes out of the
/// visible bounds, and they are always clamped to the visible bounds, so
/// that they stay inside them.
///
/// While they are stuck, the contents keep their original layout: they
/// simply float in a new position.
///
/// When the edges of the contents reach the edges of their parent, they stay
/// attached to them: they keep their original bounds, and are clipped to the
/// visible bounds instead.
///
/// Once the parent of the contents has gone out of the visible bounds, they
/// stop floating and scroll with it.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } pub use iced_widget::core::Length::Fill; }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::{column, container, scrollable, sticky, space};
/// use iced::Fill;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     scrollable(column![
///         sticky(container("I always stay in view!").width(Fill).padding(10)),
///         space().height(3000),
///     ]).into()
/// }
/// ```
pub struct Sticky<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer>
where
    Renderer: core::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Sticky<'a, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    /// Creates a new [`Sticky`] with the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Sticky<'_, Message, Theme, Renderer>
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
        self.content.as_widget_mut().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
        direction: core::Direction,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(tree, renderer, limits, direction)
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        viewport: &Rectangle,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        if !layout.bounds().is_within(viewport) {
            return;
        }

        self.content
            .as_widget_mut()
            .operate(tree, layout, viewport, renderer, operation);
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
        if !layout.bounds().is_within(viewport) {
            return;
        }

        self.content
            .as_widget_mut()
            .update(tree, event, layout, cursor, renderer, shell, viewport);
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !layout.bounds().is_within(viewport) {
            return mouse::Interaction::None;
        }

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
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if !layout.bounds().is_within(viewport) {
            return;
        }

        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Vec<overlay::Element<'b, Message, Theme, Renderer>> {
        let bounds = layout.bounds();
        let parent = layout.parent();

        if let Some(parent) = parent
            && !bounds.is_within(viewport)
            && parent.intersects(viewport)
        {
            let position = Point::new(
                stuck_axis(
                    bounds.x,
                    bounds.width,
                    viewport.x,
                    viewport.width,
                    (parent.x, parent.x + parent.width),
                ),
                stuck_axis(
                    bounds.y,
                    bounds.height,
                    viewport.y,
                    viewport.height,
                    (parent.y, parent.y + parent.height),
                ),
            );

            let layout = layout.move_to(position + translation);
            let bounds = Rectangle::new(position, bounds.size());
            let viewport = bounds.intersection(viewport).unwrap_or(bounds) + translation;

            vec![overlay::Element::new(Box::new(Overlay {
                content: &mut self.content,
                tree,
                layout,
                viewport,
            }))]
        } else {
            self.content
                .as_widget_mut()
                .overlay(tree, layout, renderer, viewport, translation)
        }
    }
}

/// Computes the position of the contents along one axis, stuck inside the
/// visible bounds and the bounds of their parent.
///
/// `position` and `size` are the natural position and size of the contents
/// along the axis, `viewport` and `viewport_size` describe the visible
/// bounds, and `parent` describes the bounds of the parent.
///
/// The returned position keeps the contents inside both the visible bounds
/// and the bounds of the parent. When that is not possible, because the
/// contents are larger than their intersection, the contents are kept
/// attached to the edge of the parent instead, so that they keep their
/// original bounds.
fn stuck_axis(
    position: f32,
    size: f32,
    viewport: f32,
    viewport_size: f32,
    parent: (f32, f32),
) -> f32 {
    let viewport_max = viewport + viewport_size - size;
    let (parent_min, parent_max) = parent;
    let parent_max = parent_max - size;

    let min = viewport.max(parent_min);
    let max = viewport_max.min(parent_max);

    if min <= max {
        position.clamp(min, max)
    } else if parent_max < viewport {
        // The contents are attached to the upper edge of the parent
        parent_max
    } else {
        // The contents are attached to the lower edge of the parent
        parent_min
    }
}

impl<'a, Message, Theme, Renderer> From<Sticky<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: core::Renderer + 'a,
{
    fn from(sticky: Sticky<'a, Message, Theme, Renderer>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(sticky)
    }
}

struct Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    content: &'b mut Element<'a, Message, Theme, Renderer>,
    tree: &'b mut widget::Tree,
    layout: Layout<'b>,
    viewport: Rectangle,
}

impl<Message, Theme, Renderer> core::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme, Renderer>
where
    Renderer: core::Renderer,
{
    fn layout(
        &mut self,
        _renderer: &Renderer,
        _bounds: Size,
        _direction: core::Direction,
    ) -> layout::Node {
        layout::Node::new(self.viewport.size()).move_to(self.viewport.position())
    }

    fn operate(
        &mut self,
        _layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            self.tree,
            self.layout,
            &self.viewport,
            renderer,
            operation,
        );
    }

    fn update(
        &mut self,
        event: &Event,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        self.content.as_widget_mut().update(
            self.tree,
            event,
            self.layout,
            cursor,
            renderer,
            shell,
            &self.viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let interaction = self.content.as_widget().mouse_interaction(
            self.tree,
            self.layout,
            cursor,
            &self.viewport,
            renderer,
        );

        if interaction == mouse::Interaction::None && cursor.is_over(self.viewport) {
            mouse::Interaction::Idle
        } else {
            interaction
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        _layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            self.layout,
            cursor,
            &self.viewport,
        );
    }

    fn overlay<'c>(
        &'c mut self,
        _layout: Layout<'c>,
        renderer: &Renderer,
    ) -> Vec<overlay::Element<'c, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            self.tree,
            self.layout,
            renderer,
            &self.viewport,
            Vector::ZERO,
        )
    }
}
