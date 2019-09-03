//! Display images in your user interface.

use crate::{
    Element, Hasher, Layout, MouseCursor, Node, Point, Rectangle, Style, Widget,
};

use std::hash::Hash;

/// A frame that displays an image while keeping aspect ratio.
///
/// It implements [`Widget`] when the associated `Renderer` implements the
/// [`image::Renderer`] trait.
///
/// [`Widget`]: ../../core/trait.Widget.html
/// [`image::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use iced::Image;
///
/// # let my_handle = String::from("some_handle");
/// let image = Image::new(my_handle);
/// ```
pub struct Image<I> {
    image: I,
    source: Option<Rectangle<u16>>,
    style: Style,
}

impl<I> std::fmt::Debug for Image<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("source", &self.source)
            .field("style", &self.style)
            .finish()
    }
}

impl<I> Image<I> {
    /// Creates a new [`Image`] with given image handle.
    ///
    /// [`Image`]: struct.Image.html
    pub fn new(image: I) -> Self {
        Image {
            image,
            source: None,
            style: Style::default().fill_width().fill_height(),
        }
    }

    /// Sets the portion of the [`Image`] to draw.
    ///
    /// [`Image`]: struct.Image.html
    pub fn clip(mut self, source: Rectangle<u16>) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the width of the [`Image`] boundaries in pixels.
    ///
    /// [`Image`]: struct.Image.html
    pub fn width(mut self, width: u32) -> Self {
        self.style = self.style.width(width);
        self
    }

    /// Sets the height of the [`Image`] boundaries in pixels.
    ///
    /// [`Image`]: struct.Image.html
    pub fn height(mut self, height: u32) -> Self {
        self.style = self.style.height(height);
        self
    }
}

impl<I, Message, Renderer> Widget<Message, Renderer> for Image<I>
where
    Renderer: self::Renderer<I>,
    I: Clone,
{
    fn node(&self, _renderer: &Renderer) -> Node {
        Node::new(self.style)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(layout.bounds(), self.image.clone(), self.source);

        MouseCursor::OutOfBounds
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.style.hash(state);
    }
}

/// The renderer of an [`Image`].
///
/// Your [renderer] will need to implement this trait before being able to use
/// an [`Image`] in your user interface.
///
/// [`Image`]: struct.Image.html
/// [renderer]: ../../renderer/index.html
pub trait Renderer<I> {
    /// Draws an [`Image`].
    ///
    /// It receives:
    ///   * the bounds of the [`Image`]
    ///   * the handle of the loaded [`Image`]
    ///   * the portion of the image to draw. If not specified, the entire image
    ///     should be drawn.
    ///
    /// [`Image`]: struct.Image.html
    fn draw(
        &mut self,
        bounds: Rectangle<f32>,
        image: I,
        source: Option<Rectangle<u16>>,
    );
}

impl<'a, I, Message, Renderer> From<Image<I>> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer<I>,
    I: Clone + 'a,
{
    fn from(image: Image<I>) -> Element<'a, Message, Renderer> {
        Element::new(image)
    }
}
