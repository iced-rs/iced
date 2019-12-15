//! Display an icon.
use crate::{layout, Element, Hasher, Layout, Length, Point, Size, Widget};

use std::{
    hash::Hash,
    path::{Path, PathBuf},
};

/// A simple icon_loader widget.
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
            height: Length::Fill,
        }
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

/// An [`Svg`] handle.
///
/// [`Svg`]: struct.Svg.html
#[derive(Debug, Clone)]
pub struct Handle {
    id: u64,
    path: PathBuf,
}

impl Handle {
    /// Creates an SVG [`Handle`] pointing to the vector image of the given
    /// path.
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn from_path<T: Into<PathBuf>>(path: T) -> Handle {
        use std::hash::Hasher as _;

        let path = path.into();

        let mut hasher = Hasher::default();
        path.hash(&mut hasher);

        Handle {
            id: hasher.finish(),
            path,
        }
    }

    /// Returns the unique identifier of the [`Handle`].
    ///
    /// [`Handle`]: struct.Handle.html
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a reference to the path of the [`Handle`].
    ///
    /// [`Handle`]: enum.Handle.html
    pub fn path(&self) -> &Path {
        &self.path
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

/// The renderer of an [`Svg`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// an [`Svg`] in your user interface.
///
/// [`Svg`]: struct.Svg.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer: crate::Renderer {
    /// Returns the default dimensions of an [`Svg`] located on the given path.
    ///
    /// [`Svg`]: struct.Svg.html
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
