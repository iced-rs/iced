//! Display images in your user interface.

use crate::{layout, Element, Hasher, Layout, Length, Point, Size, Widget};

use std::{
    hash::{Hash, Hasher as _},
    path::PathBuf,
    rc::Rc,
};

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
    handle: Handle,
    width: Length,
    height: Length,
}

impl Image {
    /// Creates a new [`Image`] with the given path.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new<T: Into<Handle>>(handle: T) -> Self {
        Image {
            handle: handle.into(),
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Sets the width of the [`Image`] boundaries.
    ///
    /// [`Image`]: struct.Image.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Image`] boundaries.
    ///
    /// [`Image`]: struct.Image.html
    pub fn height(mut self, height: Length) -> Self {
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
        let (width, height) = renderer.dimensions(&self.handle);

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
        renderer.draw(self.handle.clone(), layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.width.hash(state);
        self.height.hash(state);
    }
}

/// An [`Image`] handle.
///
/// [`Image`]: struct.Image.html
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    data: Rc<Data>,
}

impl Handle {
    /// Creates an image [`Handle`] pointing to the image of the given path.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates an image [`Handle`] containing the image data directly.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_bytes(bytes: Vec<u8>) -> Handle {
        Self::from_data(Data::Bytes(bytes))
    }

    fn from_data(data: Data) -> Handle {
        let mut hasher = Hasher::default();
        data.hash(&mut hasher);

        Handle {
            id: hasher.finish(),
            data: Rc::new(data),
        }
    }

    /// Returns the uniquie identifier of the [`Handle`].
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a reference to the image [`Data`].
    ///
    /// [`Data`]: enum.Data.html
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl From<String> for Handle {
    fn from(path: String) -> Handle {
        Handle::from_path(path)
    }
}

impl From<&str> for Handle {
    fn from(path: &str) -> Handle {
        Handle::from_path(path)
    }
}

/// The data of an [`Image`].
///
/// [`Image`]: struct.Image.html
#[derive(Debug, Clone, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    Bytes(Vec<u8>),
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
    fn dimensions(&self, handle: &Handle) -> (u32, u32);

    /// Draws an [`Image`].
    ///
    /// [`Image`]: struct.Image.html
    fn draw(&mut self, handle: Handle, layout: Layout<'_>) -> Self::Output;
}

impl<'a, Message, Renderer> From<Image> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(image: Image) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
