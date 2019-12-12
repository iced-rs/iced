//! Display an icon.
use crate::{
    image, layout, Element, Hasher, Layout, Length, Point, Rectangle, Widget,
};

use std::{
    hash::Hash,
    path::{Path, PathBuf},
};

/// A simple icon_loader widget.
#[derive(Debug, Clone)]
pub struct Icon {
    handle: image::Handle,
    size: Length,
}

impl Icon {
    /// Create a new [`Icon`] from the file at `path`.
    ///
    /// [`Icon`]: struct.Icon.html
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Icon {
            handle: image::Handle::from_path(path),
            size: Length::Fill,
        }
    }

    /// Sets the size of the [`Icon`].
    ///
    /// [`Icon`]: struct.Icon.html
    pub fn size(mut self, size: Length) -> Self {
        self.size = size;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Icon
where
    Renderer: image::Renderer,
{
    fn width(&self) -> Length {
        self.size
    }

    fn height(&self) -> Length {
        self.size
    }

    fn layout(&self, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        let mut size = limits.width(self.size).height(self.size).max();

        if size.width > size.height {
            size.width = size.height;
        } else if size.width < size.height {
            size.height = size.width;
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
        self.size.hash(state);
    }
}

impl<'a, Message, Renderer> From<Icon> for Element<'a, Message, Renderer>
where
    Renderer: image::Renderer,
{
    fn from(icon: Icon) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
