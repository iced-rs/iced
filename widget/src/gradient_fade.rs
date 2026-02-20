//! Apply a gradient opacity fade to any widget.
//!
//! The [`GradientFade`] widget wraps content and applies a smooth gradient
//! opacity transition at specified edges. This is useful for scrollable
//! content that should fade out at the edges.
//!
//! # Example
//! ```no_run
//! use iced::widget::{gradient_fade, scrollable, text, FadeEdge};
//! use iced::Element;
//!
//! fn view<'a>() -> Element<'a, ()> {
//!     gradient_fade(
//!         scrollable(text("Content that fades at the bottom")),
//!     )
//!     .edge(FadeEdge::Bottom)
//!     .height(80.0)
//!     .into()
//! }
//! ```
//!
//! # Custom Stops
//! For more control, you can specify exact fade positions:
//! ```no_run
//! use iced::widget::{gradient_fade, scrollable, text, FadeEdge};
//! use iced::Element;
//!
//! fn view<'a>() -> Element<'a, ()> {
//!     gradient_fade(
//!         scrollable(text("Content with custom fade")),
//!     )
//!     .edge(FadeEdge::Bottom)
//!     .fade_start(0.7)  // Start fading at 70% of height
//!     .fade_end(0.9)    // Fully transparent at 90% (content cut off)
//!     .into()
//! }
//! ```
use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{Element, Event, Length, Rectangle, Shell, Size, Vector, Widget};

/// Edge where the gradient fade should be applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FadeEdge {
    /// Fade at the top edge (content fades in from top).
    Top,
    /// Fade at the bottom edge (content fades out at bottom).
    #[default]
    Bottom,
    /// Fade at the left edge (content fades in from left).
    Left,
    /// Fade at the right edge (content fades out at right).
    Right,
    /// Fade at both top and bottom edges.
    Vertical,
    /// Fade at both left and right edges.
    Horizontal,
}

/// A widget that applies a gradient opacity fade to its content.
///
/// The fade creates a smooth transition from opaque to transparent at the
/// specified edge(s). This is commonly used with scrollable content to
/// indicate there's more content beyond the visible area.
///
/// # Example
/// ```no_run
/// use iced::widget::{gradient_fade, container, text};
/// use iced::Element;
///
/// fn view<'a>() -> Element<'a, ()> {
///     gradient_fade(container(text("Fading content")))
///         .height(100.0)
///         .into()
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct GradientFade<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    edge: FadeEdge,
    fade_height: Option<f32>,
    /// Custom fade start position (0.0 to 1.0, where 0.0 is top/left)
    custom_fade_start: Option<f32>,
    /// Custom fade end position (0.0 to 1.0, where 1.0 is bottom/right)
    custom_fade_end: Option<f32>,
}

impl<'a, Message, Theme, Renderer> GradientFade<'a, Message, Theme, Renderer> {
    /// Creates a new [`GradientFade`] widget with default settings.
    ///
    /// By default, fades at the bottom edge with an 80px fade height.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            edge: FadeEdge::default(),
            fade_height: Some(80.0),
            custom_fade_start: None,
            custom_fade_end: None,
        }
    }

    /// Sets which edge(s) to apply the fade to.
    pub fn edge(mut self, edge: FadeEdge) -> Self {
        self.edge = edge;
        self
    }

    /// Sets the height (or width for horizontal fades) of the fade region in pixels.
    ///
    /// Larger values create a more gradual fade effect.
    /// This is overridden if [`fade_start`] or [`fade_end`] are set.
    pub fn height(mut self, height: f32) -> Self {
        self.fade_height = Some(height.max(0.0));
        self
    }

    /// Sets the fade start position as a percentage (0.0 to 1.0).
    ///
    /// - For `Bottom` edge: 0.0 = top, 1.0 = bottom. Content is fully opaque before this point.
    /// - For `Top` edge: 0.0 = top, 1.0 = bottom. Content starts becoming opaque at this point.
    /// - For `Left`/`Right` edges: same logic but horizontal.
    ///
    /// This overrides the [`height`] setting when specified.
    pub fn fade_start(mut self, start: f32) -> Self {
        self.custom_fade_start = Some(start.clamp(0.0, 1.0));
        self.fade_height = None; // Custom stops override height
        self
    }

    /// Sets the fade end position as a percentage (0.0 to 1.0).
    ///
    /// - For `Bottom` edge: Content is fully transparent after this point.
    /// - For `Top` edge: Content is fully opaque after this point.
    /// - For `Left`/`Right` edges: same logic but horizontal.
    ///
    /// This overrides the [`height`] setting when specified.
    pub fn fade_end(mut self, end: f32) -> Self {
        self.custom_fade_end = Some(end.clamp(0.0, 1.0));
        self.fade_height = None; // Custom stops override height
        self
    }

    /// Sets both fade start and end positions at once.
    ///
    /// Convenience method for specifying custom gradient stops.
    pub fn stops(mut self, start: f32, end: f32) -> Self {
        self.custom_fade_start = Some(start.clamp(0.0, 1.0));
        self.custom_fade_end = Some(end.clamp(0.0, 1.0));
        self.fade_height = None;
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for GradientFade<'_, Message, Theme, Renderer>
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
        let bounds = layout.bounds();

        // Calculate fade positions based on custom stops or height
        let (fade_start, fade_end) = match (self.custom_fade_start, self.custom_fade_end) {
            // Both custom stops specified
            (Some(start), Some(end)) => (start, end),
            // Only start specified - end defaults to 1.0
            (Some(start), None) => (start, 1.0),
            // Only end specified - start defaults to 0.0
            (None, Some(end)) => (0.0, end),
            // No custom stops - calculate from height
            (None, None) => {
                let height = self.fade_height.unwrap_or(80.0);
                match self.edge {
                    FadeEdge::Bottom => {
                        let start = 1.0 - (height / bounds.height).min(1.0);
                        (start, 1.0)
                    }
                    FadeEdge::Top => {
                        let end = (height / bounds.height).min(1.0);
                        (0.0, end)
                    }
                    FadeEdge::Right => {
                        let start = 1.0 - (height / bounds.width).min(1.0);
                        (start, 1.0)
                    }
                    FadeEdge::Left => {
                        let end = (height / bounds.width).min(1.0);
                        (0.0, end)
                    }
                    FadeEdge::Vertical | FadeEdge::Horizontal => {
                        let dimension = if matches!(self.edge, FadeEdge::Vertical) {
                            bounds.height
                        } else {
                            bounds.width
                        };
                        let ratio = (height / dimension).min(0.5);
                        (0.0, ratio)
                    }
                }
            }
        };

        match self.edge {
            FadeEdge::Bottom => {
                // Direction 0 = TopToBottom (fade out at bottom)
                renderer.with_gradient_fade(bounds, 0, fade_start, fade_end, |renderer| {
                    self.draw_content(tree, renderer, theme, style, layout, cursor, viewport);
                });
            }
            FadeEdge::Top => {
                // Direction 1 = BottomToTop (fade out at top)
                renderer.with_gradient_fade(bounds, 1, fade_start, fade_end, |renderer| {
                    self.draw_content(tree, renderer, theme, style, layout, cursor, viewport);
                });
            }
            FadeEdge::Right => {
                // Direction 2 = LeftToRight (fade out at right)
                renderer.with_gradient_fade(bounds, 2, fade_start, fade_end, |renderer| {
                    self.draw_content(tree, renderer, theme, style, layout, cursor, viewport);
                });
            }
            FadeEdge::Left => {
                // Direction 3 = RightToLeft (fade out at left)
                renderer.with_gradient_fade(bounds, 3, fade_start, fade_end, |renderer| {
                    self.draw_content(tree, renderer, theme, style, layout, cursor, viewport);
                });
            }
            FadeEdge::Vertical => {
                // Direction 4 = VerticalBoth, fade_end = fade ratio from each edge
                renderer.with_gradient_fade(bounds, 4, fade_start, fade_end, |renderer| {
                    self.draw_content(tree, renderer, theme, style, layout, cursor, viewport);
                });
            }
            FadeEdge::Horizontal => {
                // Direction 5 = HorizontalBoth, fade_end = fade ratio from each edge
                renderer.with_gradient_fade(bounds, 5, fade_start, fade_end, |renderer| {
                    self.draw_content(tree, renderer, theme, style, layout, cursor, viewport);
                });
            }
        }
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

impl<Message, Theme, Renderer> GradientFade<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn draw_content(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<GradientFade<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(gradient_fade: GradientFade<'a, Message, Theme, Renderer>) -> Self {
        Self::new(gradient_fade)
    }
}

/// Creates a new [`GradientFade`] widget with the given content.
///
/// By default, fades at the bottom edge with an 80px fade height.
/// Use the builder methods to customize:
/// - `.edge(FadeEdge::Top)` - which edge to fade
/// - `.height(100.0)` - size of the fade region
///
/// # Example
/// ```no_run
/// use iced::widget::{gradient_fade, scrollable, text, FadeEdge};
/// use iced::Element;
///
/// fn view<'a>() -> Element<'a, ()> {
///     gradient_fade(scrollable(text("Fading content")))
///         .edge(FadeEdge::Vertical)
///         .height(60.0)
///         .into()
/// }
/// ```
pub fn gradient_fade<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> GradientFade<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    GradientFade::new(content)
}
