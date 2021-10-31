//! Display vector graphics in your application.
use crate::layout;
use crate::renderer;
use crate::svg::{self, Handle};
use crate::{Element, Hasher, Layout, Length, Point, Rectangle, Size, Widget};

use std::hash::Hash;
use std::path::PathBuf;

/// A vector graphics image.
///
/// An [`Svg`] image resizes smoothly without losing any quality.
///
/// [`Svg`] images can have a considerable rendering cost when resized,
/// specially when they are complex.
#[derive(Debug, Clone)]
pub struct Svg {
    handle: Handle,
    width: Length,
    height: Length,
}

impl Svg {
    /// Creates a new [`Svg`] from the given [`Handle`].
    pub fn new(handle: impl Into<Handle>) -> Self {
        Svg {
            handle: handle.into(),
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    /// Creates a new [`Svg`] that will display the contents of the file at the
    /// provided path.
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self::new(Handle::from_path(path))
    }

    /// Sets the width of the [`Svg`].
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Svg`].
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Svg
where
    Renderer: svg::Renderer,
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
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        renderer.draw(self.handle.clone(), layout.bounds())
    }

    fn hash_layout(&self, state: &mut Hasher) {
        std::any::TypeId::of::<Svg>().hash(state);

        self.handle.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
}

impl<'a, Message, Renderer> From<Svg> for Element<'a, Message, Renderer>
where
    Renderer: svg::Renderer,
{
    fn from(icon: Svg) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
