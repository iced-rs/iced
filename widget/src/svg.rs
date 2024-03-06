//! Display vector graphics in your application.
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::svg;
use crate::core::widget::Tree;
use crate::core::{
    Color, ContentFit, Element, Layout, Length, Rectangle, Size, Vector, Widget,
};
use crate::style::Theme;

use std::path::PathBuf;

pub use crate::core::svg::Handle;

/// A vector graphics image.
///
/// An [`Svg`] image resizes smoothly without losing any quality.
///
/// [`Svg`] images can have a considerable rendering cost when resized,
/// specially when they are complex.
#[allow(missing_debug_implementations)]
pub struct Svg<Theme = crate::Theme> {
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    style: fn(&Theme, Status) -> Appearance,
}

impl<Theme> Svg<Theme> {
    /// Creates a new [`Svg`] from the given [`Handle`].
    pub fn new(handle: impl Into<Handle>) -> Self
    where
        Theme: Style,
    {
        Svg {
            handle: handle.into(),
            width: Length::Fill,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            style: Theme::style(),
        }
    }

    /// Creates a new [`Svg`] that will display the contents of the file at the
    /// provided path.
    #[must_use]
    pub fn from_path(path: impl Into<PathBuf>) -> Self
    where
        Theme: Style,
    {
        Self::new(Handle::from_path(path))
    }

    /// Sets the width of the [`Svg`].
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Svg`].
    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`ContentFit`] of the [`Svg`].
    ///
    /// Defaults to [`ContentFit::Contain`]
    #[must_use]
    pub fn content_fit(self, content_fit: ContentFit) -> Self {
        Self {
            content_fit,
            ..self
        }
    }

    /// Sets the style variant of this [`Svg`].
    #[must_use]
    pub fn style(mut self, style: fn(&Theme, Status) -> Appearance) -> Self {
        self.style = style.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Svg<Theme>
where
    Renderer: svg::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // The raw w/h of the underlying image
        let Size { width, height } = renderer.dimensions(&self.handle);
        let image_size = Size::new(width as f32, height as f32);

        // The size to be available to the widget prior to `Shrink`ing
        let raw_size = limits.resolve(self.width, self.height, image_size);

        // The uncropped size of the image when fit to the bounds above
        let full_size = self.content_fit.fit(image_size, raw_size);

        // Shrink the widget to fit the resized image, if requested
        let final_size = Size {
            width: match self.width {
                Length::Shrink => f32::min(raw_size.width, full_size.width),
                _ => raw_size.width,
            },
            height: match self.height {
                Length::Shrink => f32::min(raw_size.height, full_size.height),
                _ => raw_size.height,
            },
        };

        layout::Node::new(final_size)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let Size { width, height } = renderer.dimensions(&self.handle);
        let image_size = Size::new(width as f32, height as f32);

        let bounds = layout.bounds();
        let adjusted_fit = self.content_fit.fit(image_size, bounds.size());
        let is_mouse_over = cursor.is_over(bounds);

        let render = |renderer: &mut Renderer| {
            let offset = Vector::new(
                (bounds.width - adjusted_fit.width).max(0.0) / 2.0,
                (bounds.height - adjusted_fit.height).max(0.0) / 2.0,
            );

            let drawing_bounds = Rectangle {
                width: adjusted_fit.width,
                height: adjusted_fit.height,
                ..bounds
            };

            let status = if is_mouse_over {
                Status::Hovered
            } else {
                Status::Idle
            };

            let appearance = (self.style)(theme, status);

            renderer.draw(
                self.handle.clone(),
                appearance.color,
                drawing_bounds + offset,
            );
        };

        if adjusted_fit.width > bounds.width
            || adjusted_fit.height > bounds.height
        {
            renderer.with_layer(bounds, render);
        } else {
            render(renderer);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Svg<Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: svg::Renderer + 'a,
{
    fn from(icon: Svg<Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(icon)
    }
}

/// The possible status of an [`Svg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Svg`] is idle.
    Idle,
    /// The [`Svg`] is being hovered.
    Hovered,
}

/// The appearance of an [`Svg`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Appearance {
    /// The [`Color`] filter of an [`Svg`].
    ///
    /// Useful for coloring a symbolic icon.
    ///
    /// `None` keeps the original color.
    pub color: Option<Color>,
}

/// The definiton of the default style of an [`Svg`].
pub trait Style {
    /// Returns the default style of an [`Svg`].
    fn style() -> fn(&Self, Status) -> Appearance;
}

impl Style for Theme {
    fn style() -> fn(&Self, Status) -> Appearance {
        |_, _| Appearance::default()
    }
}
