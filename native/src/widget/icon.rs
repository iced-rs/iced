//! Display an icon.
use crate::{layout, Element, Hasher, Layout, Length, Point, Rectangle, Widget};

use std::hash::Hash;
use std::path::{Path, PathBuf};

/// A simple icon_loader widget.
#[derive(Debug, Clone)]
pub struct Icon {
    path: PathBuf,
    size: Length,
}

impl Icon {
    /// Create a new [`Icon`] from the file at `path`.
    ///
    /// [`Icon`]: struct.Icon.html
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Icon {
            path: path.into(),
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
    Renderer: self::Renderer,
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
        let bounds = layout.bounds();

        renderer.draw(
            bounds,
            self.path.as_path(),
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.size.hash(state);
    }
}

/// The renderer of an [`Icon`].
///
/// Your [renderer] will need to implement this trait before being
/// able to use [`Icon`] in your [`UserInterface`].
///
/// [`Icon`]: struct.Icon.html
/// [renderer]: ../../renderer/index.html
/// [`UserInterface`]: ../../struct.UserInterface.html
pub trait Renderer: crate::Renderer {
    /// Draws an [`Icon`].
    ///
    /// [`Icon`]: struct.Icon.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        path: &Path,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Icon> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(icon: Icon) -> Element<'a, Message, Renderer> {
        Element::new(icon)
    }
}
