//! Display images in your user interface.
use crate::layout;
use crate::{Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget};

use std::{
    hash::{Hash, Hasher as _},
    path::PathBuf,
    sync::Arc,
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
#[derive(Debug, Hash)]
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

        let mut size = limits
            .width(self.width)
            .height(self.height)
            .resolve(Size::new(width as f32, height as f32));

        let viewport_aspect_ratio = size.width / size.height;

        if viewport_aspect_ratio > aspect_ratio {
            size.width = width as f32 * size.height / height as f32;
        } else {
            size.height = height as f32 * size.width / width as f32;
        }

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(self.handle.clone(), layout)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.handle.hash(state);
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
    data: Arc<Data>,
}

impl Handle {
    /// Creates an image [`Handle`] pointing to the image of the given path.
    ///
    /// Makes an educated guess about the image format by examining the data in the file.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates an image [`Handle`] containing the image pixels directly. This
    /// function expects the input data to be provided as a `Vec<u8>` of BGRA
    /// pixels.
    ///
    /// This is useful if you have already decoded your image.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<u8>) -> Handle {
        Self::from_data(Data::Pixels {
            width,
            height,
            pixels,
        })
    }

    /// Creates an image [`Handle`] containing the image data directly.
    ///
    /// Makes an educated guess about the image format by examining the given data.
    ///
    /// This is useful if you already have your image loaded in-memory, maybe
    /// because you downloaded or generated it procedurally.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_memory(bytes: Vec<u8>) -> Handle {
        Self::from_data(Data::Bytes(bytes))
    }

    fn from_data(data: Data) -> Handle {
        let mut hasher = Hasher::default();
        data.hash(&mut hasher);

        Handle {
            id: hasher.finish(),
            data: Arc::new(data),
        }
    }

    /// Returns the unique identifier of the [`Handle`].
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

impl<T> From<T> for Handle
where
    T: Into<PathBuf>,
{
    fn from(path: T) -> Handle {
        Handle::from_path(path.into())
    }
}

impl Hash for Handle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// The data of an [`Image`].
///
/// [`Image`]: struct.Image.html
#[derive(Clone, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    Bytes(Vec<u8>),

    /// Decoded image pixels in BGRA format.
    Pixels {
        /// The width of the image.
        width: u32,
        /// The height of the image.
        height: u32,
        /// The pixels.
        pixels: Vec<u8>,
    },
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({:?})", path),
            Data::Bytes(_) => write!(f, "Bytes(...)"),
            Data::Pixels { width, height, .. } => {
                write!(f, "Pixels({} * {})", width, height)
            }
        }
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
