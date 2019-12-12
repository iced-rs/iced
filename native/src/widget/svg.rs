//! Display an icon.
use crate::{
    image, layout, Element, Hasher, Layout, Length, Point, Size, Widget,
};

use std::{
    hash::Hash,
    path::PathBuf,
};

/// A simple icon_loader widget.
#[derive(Debug, Clone)]
pub struct Svg {
    handle: image::Handle,
    width: Length,
    height: Length,
}

impl Svg {
    /// Create a new [`Svg`] from the file at `path`.
    ///
    /// [`Svg`]: struct.Svg.html
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Svg {
            handle: image::Handle::from_path(path),
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
    Renderer: image::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let (width, height) = renderer.dimensions(&self.handle);

        let aspect_ratio = width as f32 / height as f32;

        let mut size = limits
            .width(self.width)
            .height(self.height)
            .max();

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

impl<'a, Message, Renderer> From<Svg> for Element<'a, Message, Renderer>
where
    Renderer: image::Renderer,
{
    fn from(icon: Svg) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
