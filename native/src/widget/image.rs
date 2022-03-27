//! Display images in your user interface.
pub mod viewer;
pub use viewer::Viewer;

use crate::image;
use crate::image_filter;
use crate::layout;
use crate::renderer;
use crate::{
    ContentFit, Element, Layout, Length, Point, Rectangle, Size, Vector, Widget,
};

use std::hash::Hash;

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```
/// # use iced_native::widget::Image;
/// # use iced_native::image;
/// #
/// let image = Image::<image::Handle>::new("resources/ferris.png");
/// ```
///
/// <img src="https://github.com/iced-rs/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
#[derive(Debug, Hash)]
pub struct Image<Handle> {
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    filters: image_filter::FilterOptions,
}

impl<Handle> Image<Handle> {
    /// Creates a new [`Image`] with the given path.
    pub fn new<T: Into<Handle>>(handle: T) -> Self {
        Image {
            handle: handle.into(),
            width: Length::Shrink,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            filters: image_filter::FilterOptions { mag_filter: image_filter::ImageFilter::Linear, min_filter: image_filter::ImageFilter::Linear, },
        }
    }

    /// Sets the width of the [`Image`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Image`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`ContentFit`] of the [`Image`].
    ///
    /// Defaults to [`ContentFit::Contain`]
    pub fn content_fit(self, content_fit: ContentFit) -> Self {
        Self {
            content_fit,
            ..self
        }
    }
    
    /// Sets the filtering option to use when rendering a down-scaled (IE zoomed-out) image
    ///
    /// Default is `Linear`
    pub fn min_filter(mut self, min_filter: image_filter::ImageFilter) -> Self {
        self.filters.min_filter = min_filter;
        self
    }

    /// Sets the filtering option to use when rendering a up-scaled (IE zoomed-in) image
    ///
    /// Default is `Linear`
    pub fn mag_filter(mut self, mag_filter: image_filter::ImageFilter) -> Self {
        self.filters.mag_filter = mag_filter;
        self
    }
}

/// Computes the layout of an [`Image`].
pub fn layout<Renderer, Handle>(
    renderer: &Renderer,
    limits: &layout::Limits,
    handle: &Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
) -> layout::Node
where
    Renderer: image::Renderer<Handle = Handle>,
{
    // The raw w/h of the underlying image
    let image_size = {
        let (width, height) = renderer.dimensions(handle);

        Size::new(width as f32, height as f32)
    };

    // The size to be available to the widget prior to `Shrink`ing
    let raw_size = limits.width(width).height(height).resolve(image_size);

    // The uncropped size of the image when fit to the bounds above
    let full_size = content_fit.fit(image_size, raw_size);

    // Shrink the widget to fit the resized image, if requested
    let final_size = Size {
        width: match width {
            Length::Shrink => f32::min(raw_size.width, full_size.width),
            _ => raw_size.width,
        },
        height: match height {
            Length::Shrink => f32::min(raw_size.height, full_size.height),
            _ => raw_size.height,
        },
    };

    layout::Node::new(final_size)
}

impl<Message, Renderer, Handle> Widget<Message, Renderer> for Image<Handle>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + Hash,
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
        layout(
            renderer,
            limits,
            &self.handle,
            self.width,
            self.height,
            self.content_fit,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        let (width, height) = renderer.dimensions(&self.handle);
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

            renderer.draw(self.handle.clone(), drawing_bounds + offset, self.filters)
        };

        if adjusted_fit.width > bounds.width
            || adjusted_fit.height > bounds.height
        {
            renderer.with_layer(bounds, render);
        } else {
            render(renderer)
        }
    }
}

impl<'a, Message, Renderer, Handle> From<Image<Handle>>
    for Element<'a, Message, Renderer>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + Hash + 'a,
{
    fn from(image: Image<Handle>) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
