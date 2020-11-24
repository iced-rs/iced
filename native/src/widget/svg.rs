//! Display vector graphics in your application.
use crate::layout;
use crate::{Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget};

use std::{
    hash::{Hash, Hasher as _},
    path::PathBuf,
    sync::Arc,
};

/// A vector graphics image.
///
/// An [`Svg`] image resizes smoothly without losing any quality.
///
/// [`Svg`] images can have a considerable rendering cost when resized,
/// specially when they are complex.
///
/// [`Svg`]: struct.Svg.html
#[derive(Debug, Clone)]
pub struct Svg {
    handle: Handle,
    width: Length,
    height: Length,
}

impl Svg {
    /// Creates a new [`Svg`] from the given [`Handle`].
    ///
    /// [`Svg`]: struct.Svg.html
    /// [`Handle`]: struct.Handle.html
    pub fn new(handle: impl Into<Handle>) -> Self {
        Svg {
            handle: handle.into(),
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    /// Creates a new [`Svg`] that will display the contents of the file at the
    /// provided path.
    ///
    /// [`Svg`]: struct.Svg.html
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self::new(Handle::from_path(path))
    }

    /// Sets the width of the [`Svg`].
    ///
    /// [`Svg`]: struct.Svg.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Svg`].
    ///
    /// [`Svg`]: struct.Svg.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Svg
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
        std::any::TypeId::of::<Svg>().hash(state);

        self.handle.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

/// An [`Svg`] handle.
///
/// [`Svg`]: struct.Svg.html
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    data: Arc<Data>,
}

impl Handle {
    /// Creates an SVG [`Handle`] pointing to the vector image of the given
    /// path.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path(path: impl Into<PathBuf>) -> Handle {
        Self::from_data(Data::Path(path.into()))
    }

    /// Creates an SVG [`Handle`] from raw bytes containing either an SVG string
    /// or gzip compressed data.
    ///
    /// This is useful if you already have your SVG data in-memory, maybe
    /// because you downloaded or generated it procedurally.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_memory(bytes: impl Into<Vec<u8>>) -> Handle {
        Self::from_data(Data::Bytes(bytes.into()))
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

    /// Returns a reference to the SVG [`Data`].
    ///
    /// [`Data`]: enum.Data.html
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl Hash for Handle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// The data of an [`Svg`].
///
/// [`Svg`]: struct.Svg.html
#[derive(Clone, Hash)]
pub enum Data {
    /// File data
    Path(PathBuf),

    /// In-memory data
    ///
    /// Can contain an SVG string or a gzip compressed data.
    Bytes(Vec<u8>),
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Path(path) => write!(f, "Path({:?})", path),
            Data::Bytes(_) => write!(f, "Bytes(...)"),
        }
    }
}

/// The renderer of an [`Svg`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// an [`Svg`] in your user interface.
///
/// [`Svg`]: struct.Svg.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Returns the default dimensions of an [`Svg`] for the given [`Handle`].
    ///
    /// [`Svg`]: struct.Svg.html
    /// [`Handle`]: struct.Handle.html
    fn dimensions(&self, handle: &Handle) -> (u32, u32);

    /// Draws an [`Svg`].
    ///
    /// [`Svg`]: struct.Svg.html
    fn draw(&mut self, handle: Handle, layout: Layout<'_>) -> Self::Output;
}

impl<'a, Message, Renderer> From<Svg> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(icon: Svg) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
