//! Display vector graphics in your application.
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::svg;
use crate::core::widget::Tree;
use crate::core::{
    Color, ContentFit, Element, Layout, Length, Rectangle, Size, Theme, Vector,
    Widget,
};

use std::path::PathBuf;

pub use crate::core::svg::Handle;

/// A vector graphics image.
///
/// An [`Svg`] image resizes smoothly without losing any quality.
///
/// [`Svg`] images can have a considerable rendering cost when resized,
/// specially when they are complex.
#[allow(missing_debug_implementations)]
pub struct Svg<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Svg<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a new [`Svg`] from the given [`Handle`].
    pub fn new(handle: impl Into<Handle>) -> Self {
        Svg {
            handle: handle.into(),
            width: Length::Fill,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            class: Theme::default(),
        }
    }

    /// Creates a new [`Svg`] that will display the contents of the file at the
    /// provided path.
    #[must_use]
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
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

    /// Sets the style of the [`Svg`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Svg`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Svg<'a, Theme>
where
    Renderer: svg::Renderer,
    Theme: Catalog,
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

            let style = theme.style(&self.class, status);

            renderer.draw(
                self.handle.clone(),
                style.color,
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

impl<'a, Message, Theme, Renderer> From<Svg<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: svg::Renderer + 'a,
{
    fn from(icon: Svg<'a, Theme>) -> Element<'a, Message, Theme, Renderer> {
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
pub struct Style {
    /// The [`Color`] filter of an [`Svg`].
    ///
    /// Useful for coloring a symbolic icon.
    ///
    /// `None` keeps the original color.
    pub color: Option<Color>,
}

/// The theme catalog of an [`Svg`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme, _status| Style::default())
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// A styling function for an [`Svg`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl<'a, Theme> From<Style> for StyleFn<'a, Theme> {
    fn from(style: Style) -> Self {
        Box::new(move |_theme, _status| style)
    }
}
