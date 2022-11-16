//! Display vector graphics in your application.
use crate::layout;
use crate::renderer;
use crate::svg;
use crate::widget::Tree;
use crate::{
    ContentFit, Element, Layout, Length, Point, Rectangle, Size, Vector, Widget,
};

use std::path::PathBuf;

pub use svg::{Handle, StyleSheet};

/// A vector graphics image.
///
/// An [`Svg`] image resizes smoothly without losing any quality.
///
/// [`Svg`] images can have a considerable rendering cost when resized,
/// specially when they are complex.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Svg<Renderer>
where
    Renderer: svg::Renderer,
    Renderer::Theme: StyleSheet,
{
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<Renderer> Svg<Renderer>
where
    Renderer: svg::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Creates a new [`Svg`] from the given [`Handle`].
    pub fn new(handle: impl Into<Handle>) -> Self {
        Svg {
            handle: handle.into(),
            width: Length::Fill,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            style: Default::default(),
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
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Svg`].
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
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
    pub fn style(
        mut self,
        style: <Renderer::Theme as StyleSheet>::Style,
    ) -> Self {
        self.style = style;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Svg<Renderer>
where
    Renderer: svg::Renderer,
    Renderer::Theme: iced_style::svg::StyleSheet,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // The raw w/h of the underlying image
        let Size { width, height } = renderer.dimensions(&self.handle);
        let image_size = Size::new(width as f32, height as f32);

        // The size to be available to the widget prior to `Shrink`ing
        let raw_size = limits
            .width(self.width)
            .height(self.height)
            .resolve(image_size);

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
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let mut handle = self.handle.clone();
        handle.set_appearance(theme.appearance(self.style));

        let Size { width, height } = renderer.dimensions(&self.handle);
        let image_size = Size::new(width as f32, height as f32);

        let bounds = layout.bounds();
        let adjusted_fit = self.content_fit.fit(image_size, bounds.size());

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

            renderer.draw(handle, drawing_bounds + offset);
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

impl<'a, Message, Renderer> From<Svg<Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: svg::Renderer + 'a,
    Renderer::Theme: iced_style::svg::StyleSheet,
{
    fn from(icon: Svg<Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
