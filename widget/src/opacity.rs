//! Apply opacity to any widget.
//!
//! The [`Opacity`] widget wraps content and applies transparency to it.
//! Nested opacity widgets multiply their values (e.g., 50% × 50% = 25%).
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{Clipboard, Element, Event, Length, Rectangle, Shell, Size, Vector, Widget};

/// A widget that applies opacity to its content.
///
/// Opacity values range from `0.0` (fully transparent) to `1.0` (fully opaque).
/// Nested [`Opacity`] widgets multiply their values together.
///
/// # Example
/// ```no_run
/// use iced::widget::{opacity, container, text};
/// use iced::Element;
///
/// fn view<'a>() -> Element<'a, ()> {
///     opacity(0.5, container(text("50% transparent"))).into()
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Opacity<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    opacity: f32,
    content: Element<'a, Message, Theme, Renderer>,
}

impl<'a, Message, Theme, Renderer> Opacity<'a, Message, Theme, Renderer> {
    /// Creates a new [`Opacity`] widget with the given opacity value and content.
    ///
    /// The opacity is clamped to the range `[0.0, 1.0]`.
    pub fn new(opacity: f32, content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            opacity: opacity.clamp(0.0, 1.0),
            content: content.into(),
        }
    }

    /// Sets the opacity value.
    ///
    /// The opacity is clamped to the range `[0.0, 1.0]`.
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Opacity<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
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
        renderer.with_opacity(layout.bounds(), self.opacity, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Opacity<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(opacity: Opacity<'a, Message, Theme, Renderer>) -> Self {
        Self::new(opacity)
    }
}

/// Creates a new [`Opacity`] widget with the given opacity value and content.
///
/// Opacity values range from `0.0` (fully transparent) to `1.0` (fully opaque).
///
/// # Example
/// ```no_run
/// use iced::widget::{opacity, text};
/// use iced::Element;
///
/// fn view<'a>() -> Element<'a, ()> {
///     opacity(0.5, text("50% transparent")).into()
/// }
/// ```
pub fn opacity<'a, Message, Theme, Renderer>(
    opacity: f32,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Opacity<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    Opacity::new(opacity, content)
}
