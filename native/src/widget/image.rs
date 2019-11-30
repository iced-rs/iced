//! Display images in your user interface.

use crate::{layout, Element, Hasher, Layout, Length, Point, Size, Widget};

use std::hash::Hash;

/// A frame that displays an image while keeping aspect ratio.
///
/// # Example
///
/// ```
/// # use iced_native::Image;
/// #
/// let image = Image::new("resources/ferris.png");
/// ```
///
/// <img src="https://github.com/hecrj/iced/blob/9712b319bb7a32848001b96bd84977430f14b623/examples/resources/ferris.png?raw=true" width="300">
#[derive(Debug)]
pub struct Image {
    path: String,
    width: Length,
    height: Length,
}

impl Image {
    /// Creates a new [`Image`] with the given path.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new<T: Into<String>>(path: T) -> Self {
        Image {
            path: path.into(),
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the width of the [`Image`] boundaries.
    ///
    /// [`Image`]: struct.Image.html
    pub const fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Image`] boundaries.
    ///
    /// [`Image`]: struct.Image.html
    pub const fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Image
where
    Renderer: self::Renderer,
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
        let (width, height) = renderer.dimensions(&self.path);

        let aspect_ratio = width as f32 / height as f32;

        // TODO: Deal with additional cases
        let (width, height) = match (self.width, self.height) {
            (Length::Units(width), _) => (
                self.width,
                Length::Units((width as f32 / aspect_ratio).round() as u16),
            ),
            (_, _) => {
                (Length::Units(width as u16), Length::Units(height as u16))
            }
        };

        let mut size = limits.width(width).height(height).resolve(Size::ZERO);

        size.height = size.width / aspect_ratio;

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(&self.path, layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
        self.height.hash(state);
    }
}

/// The renderer of an [`Image`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// an [`Image`] in your user interface.
///
/// [`Image`]: struct.Image.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Returns the dimensions of an [`Image`] located on the given path.
    ///
    /// [`Image`]: struct.Image.html
    fn dimensions(&self, path: &str) -> (u32, u32);

    /// Draws an [`Image`].
    ///
    /// [`Image`]: struct.Image.html
    fn draw(&mut self, path: &str, layout: Layout<'_>) -> Self::Output;
}

impl<'a, Message, Renderer> From<Image> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(image: Image) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
