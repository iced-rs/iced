//! Display images in your user interface.
pub mod viewer;
pub use viewer::Viewer;

use crate::image;
use crate::layout;
use crate::renderer;
use crate::{Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget};

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
}

impl<Handle> Image<Handle> {
    /// Creates a new [`Image`] with the given path.
    pub fn new<T: Into<Handle>>(handle: T) -> Self {
        Image {
            handle: handle.into(),
            width: Length::Shrink,
            height: Length::Shrink,
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
}

/// Computes the layout of an [`Image`].
pub fn layout<Renderer, Handle>(
    renderer: &Renderer,
    limits: &layout::Limits,
    handle: &Handle,
    width: Length,
    height: Length,
) -> layout::Node
where
    Renderer: image::Renderer<Handle = Handle>,
{
    let (original_width, original_height) = renderer.dimensions(handle);

    let mut size = limits
        .width(width)
        .height(height)
        .resolve(Size::new(original_width as f32, original_height as f32));

    let aspect_ratio = original_width as f32 / original_height as f32;
    let viewport_aspect_ratio = size.width / size.height;

    if viewport_aspect_ratio > aspect_ratio {
        size.width =
            original_width as f32 * size.height / original_height as f32;
    } else {
        size.height =
            original_height as f32 * size.width / original_width as f32;
    }

    layout::Node::new(size)
}

/// Hashes the layout attributes of an [`Image`].
pub fn hash_layout<Handle: Hash>(
    state: &mut Hasher,
    handle: &Handle,
    width: Length,
    height: Length,
) {
    struct Marker;
    std::any::TypeId::of::<Marker>().hash(state);

    handle.hash(state);
    width.hash(state);
    height.hash(state);
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
        layout(renderer, limits, &self.handle, self.width, self.height)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        renderer.draw(self.handle.clone(), layout.bounds());
    }

    fn hash_layout(&self, state: &mut Hasher) {
        hash_layout(state, &self.handle, self.width, self.height)
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
