//! Display images in your user interface.
pub mod viewer;
use iced_renderer::core::{Point, RotationLayout};
pub use viewer::Viewer;

use crate::core::image;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    ContentFit, Element, Layout, Length, Rectangle, Size, Widget,
};

pub use image::{FilterMethod, Handle};

/// Creates a new [`Viewer`] with the given image `Handle`.
pub fn viewer<Handle>(handle: Handle) -> Viewer<Handle> {
    Viewer::new(handle)
}

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```no_run
/// # use iced_widget::image::{self, Image};
/// #
/// let image = Image::<image::Handle>::new("resources/ferris.png");
/// ```
///
/// <img src="https://github.com/iced-rs/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
#[derive(Debug)]
pub struct Image<Handle> {
    handle: Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    filter_method: FilterMethod,
    rotation: f32,
    rotation_layout: RotationLayout,
}

impl<Handle> Image<Handle> {
    /// Creates a new [`Image`] with the given path.
    pub fn new<T: Into<Handle>>(handle: T) -> Self {
        Image {
            handle: handle.into(),
            width: Length::Shrink,
            height: Length::Shrink,
            content_fit: ContentFit::Contain,
            filter_method: FilterMethod::default(),
            rotation: 0.0,
            rotation_layout: RotationLayout::Change,
        }
    }

    /// Sets the width of the [`Image`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Image`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`ContentFit`] of the [`Image`].
    ///
    /// Defaults to [`ContentFit::Contain`]
    pub fn content_fit(mut self, content_fit: ContentFit) -> Self {
        self.content_fit = content_fit;
        self
    }

    /// Sets the [`FilterMethod`] of the [`Image`].
    pub fn filter_method(mut self, filter_method: FilterMethod) -> Self {
        self.filter_method = filter_method;
        self
    }

    /// Rotates the [`Image`] by the given angle in radians.
    pub fn rotation(mut self, degrees: f32) -> Self {
        self.rotation = degrees;
        self
    }

    /// Sets the [`RotationLayout`] of the [`Image`].
    pub fn rotation_layout(mut self, rotation_layout: RotationLayout) -> Self {
        self.rotation_layout = rotation_layout;
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
    rotation: f32,
    rotation_layout: RotationLayout,
) -> layout::Node
where
    Renderer: image::Renderer<Handle = Handle>,
{
    // The raw w/h of the underlying image
    let image_size = renderer.measure_image(handle);
    let image_size =
        Size::new(image_size.width as f32, image_size.height as f32);

    // The rotated size of the image
    let rotated_size = rotation_layout.apply_to_size(image_size, rotation);

    // The size to be available to the widget prior to `Shrink`ing
    let raw_size = limits.resolve(width, height, rotated_size);

    // The uncropped size of the image when fit to the bounds above
    let full_size = content_fit.fit(rotated_size, raw_size);

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

/// Draws an [`Image`]
pub fn draw<Renderer, Handle>(
    renderer: &mut Renderer,
    layout: Layout<'_>,
    handle: &Handle,
    content_fit: ContentFit,
    filter_method: FilterMethod,
    rotation: f32,
    rotation_layout: RotationLayout,
) where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
{
    let Size { width, height } = renderer.measure_image(handle);
    let image_size = Size::new(width as f32, height as f32);
    let rotated_size = rotation_layout.apply_to_size(image_size, rotation);

    let bounds = layout.bounds();
    let adjusted_fit = content_fit.fit(rotated_size, bounds.size());
    let scale = Size::new(
        adjusted_fit.width / rotated_size.width,
        adjusted_fit.height / rotated_size.height,
    );

    let render = |renderer: &mut Renderer| {
        let position = match content_fit {
            ContentFit::None => Point::new(
                bounds.position().x
                    + (rotated_size.width - image_size.width) / 2.0,
                bounds.position().y
                    + (rotated_size.height - image_size.height) / 2.0,
            ),
            _ => Point::new(
                bounds.center_x() - image_size.width / 2.0,
                bounds.center_y() - image_size.height / 2.0,
            ),
        };

        let drawing_bounds = Rectangle::new(position, image_size);

        renderer.draw_image(
            handle.clone(),
            filter_method,
            drawing_bounds,
            rotation,
            scale,
        );
    };

    if adjusted_fit.width > bounds.width || adjusted_fit.height > bounds.height
    {
        renderer.with_layer(bounds, render);
    } else {
        render(renderer);
    }
}

impl<Message, Theme, Renderer, Handle> Widget<Message, Theme, Renderer>
    for Image<Handle>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone,
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
        layout(
            renderer,
            limits,
            &self.handle,
            self.width,
            self.height,
            self.content_fit,
            self.rotation,
            self.rotation_layout,
        )
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            layout,
            &self.handle,
            self.content_fit,
            self.filter_method,
            self.rotation,
            self.rotation_layout,
        );
    }
}

impl<'a, Message, Theme, Renderer, Handle> From<Image<Handle>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = Handle>,
    Handle: Clone + 'a,
{
    fn from(image: Image<Handle>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(image)
    }
}
