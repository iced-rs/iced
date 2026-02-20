//! Apply a backdrop blur effect to any widget.
//!
//! The [`BackdropBlur`] widget wraps content and applies a blur effect to
//! whatever is rendered behind it, similar to CSS `backdrop-filter: blur()`.
//!
//! # Example
//! ```no_run
//! use iced::widget::{backdrop_blur, container, text};
//! use iced::Element;
//!
//! fn view<'a>() -> Element<'a, ()> {
//!     backdrop_blur(
//!         container(text("Content on top of blur")),
//!     )
//!     .blur_radius(20.0)
//!     .into()
//! }
//! ```
//!
//! # Notes
//! - The blur effect is applied to content that was rendered BEFORE the blur widget
//! - Content inside the blur widget is rendered on top of the blurred background
//! - Larger blur radii are more expensive to render

use crate::core::layout::{self, Layout};
use crate::core::mouse;
use crate::core::overlay;
use crate::core::renderer;
use crate::core::widget::Operation;
use crate::core::widget::tree::{self, Tree};
use crate::core::{Element, Event, Length, Rectangle, Shell, Size, Vector, Widget};

/// A widget that applies a backdrop blur effect to content behind it.
///
/// The blur effect is applied to whatever was rendered before this widget,
/// and the widget's content is drawn on top of the blurred region.
///
/// # Example
/// ```no_run
/// use iced::widget::{backdrop_blur, container, text};
/// use iced::Element;
///
/// fn view<'a>() -> Element<'a, ()> {
///     backdrop_blur(container(text("Glass panel")))
///         .blur_radius(20.0)
///         .into()
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct BackdropBlur<'a, Message, Theme = crate::Theme, Renderer = crate::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    /// Blur radius in logical pixels
    blur_radius: f32,
    /// Border radius [top_left, top_right, bottom_right, bottom_left] in logical pixels
    border_radius: [f32; 4],
    width: Length,
    height: Length,
}

impl<'a, Message, Theme, Renderer> BackdropBlur<'a, Message, Theme, Renderer> {
    /// Creates a new [`BackdropBlur`] widget with default settings.
    ///
    /// By default, uses a 10px blur radius.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            blur_radius: 10.0,
            border_radius: [0.0; 4],
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the blur radius in logical pixels.
    ///
    /// Larger values create a stronger blur effect but may be more expensive to render.
    /// A value of 0 effectively disables the blur.
    pub fn blur_radius(mut self, radius: f32) -> Self {
        self.blur_radius = radius.max(0.0);
        self
    }

    /// Sets the border radius for all corners uniformly.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = [radius; 4];
        self
    }

    /// Sets the border radius for each corner individually.
    /// Order: [top_left, top_right, bottom_right, bottom_left]
    pub fn border_radius_corners(mut self, radii: [f32; 4]) -> Self {
        self.border_radius = radii;
        self
    }

    /// Sets the width of the widget.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the widget.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for BackdropBlur<'_, Message, Theme, Renderer>
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
        tree.diff_children(&[&self.content]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let content = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits);

        let size = limits.resolve(self.width, self.height, content.size());

        layout::Node::with_children(size, vec![content])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
            layout.children().next().unwrap(),
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
            layout.children().next().unwrap(),
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

        // Only use the post-blur layer mechanism when we actually have a meaningful blur effect.
        // For blur_radius < 3.0, the W3C box blur formula produces very few samples (box_size < 6),
        // resulting in negligible visual blur while still incurring full pipeline overhead.
        if self.blur_radius >= 3.0 {
            // Draw the backdrop blur effect at this location
            // This blurs whatever was rendered before this widget
            renderer.draw_backdrop_blur(bounds, self.blur_radius, self.border_radius);

            // Draw the content in a post-blur layer so it appears ON TOP of the blur
            // This ensures the blur widget's children are rendered after the blur effect is applied
            renderer.start_post_blur_layer(bounds);
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
            renderer.end_post_blur_layer();
        } else {
            // No blur - just draw children normally
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
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
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<BackdropBlur<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: renderer::Renderer + 'a,
{
    fn from(backdrop_blur: BackdropBlur<'a, Message, Theme, Renderer>) -> Self {
        Self::new(backdrop_blur)
    }
}

/// Creates a new [`BackdropBlur`] widget.
///
/// The widget blurs whatever is behind it and draws its content on top.
///
/// # Example
/// ```no_run
/// use iced::widget::{backdrop_blur, container, text};
/// use iced::Element;
///
/// fn view<'a>() -> Element<'a, ()> {
///     backdrop_blur(container(text("Glass effect")))
///         .blur_radius(15.0)
///         .into()
/// }
/// ```
pub fn backdrop_blur<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> BackdropBlur<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    BackdropBlur::new(content)
}
