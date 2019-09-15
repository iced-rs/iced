//! Display images in your user interface.

use crate::{
    Align, Element, Hasher, Layout, MouseCursor, Node, Point, Rectangle, Style,
    Widget,
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
    /// The image handle
    pub image: I,
    source: Option<Rectangle<u16>>,
    /// The width of the image
    pub width: Option<u16>,
    height: Option<u16>,
    style: Style,
}

impl<I> std::fmt::Debug for Image<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Image")
            .field("source", &self.source)
            .field("width", &self.width)
            .field("height", &self.height)
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
            width: None,
            height: None,
            style: Style::default(),
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
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets the height of the [`Image`] boundaries in pixels.
    ///
    /// [`Image`]: struct.Image.html
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Sets the alignment of the [`Image`] itself.
    ///
    /// This is useful if you want to override the default alignment given by
    /// the parent container.
    ///
    /// [`Image`]: struct.Image.html
    pub fn align_self(mut self, align: Align) -> Self {
        self.style = self.style.align_self(align);
        self
    }
}

impl<I, Message, Renderer> Widget<Message, Renderer> for Image<I>
where
    Renderer: self::Renderer<I>,
    I: Clone,
{
    fn node(&self, renderer: &Renderer) -> Node {
        renderer.node(
            self.style,
            &self.image,
            self.width,
            self.height,
            self.source,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(&self.image, layout.bounds(), self.source);

        MouseCursor::OutOfBounds
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.style.hash(state);
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
pub trait Renderer<I> {
    /// Creates a [`Node`] with the given [`Style`] for the provided [`Image`]
    /// and its size.
    ///
    /// You should probably keep the original aspect ratio, if possible.
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Style`]: ../../struct.Style.html
    /// [`Image`]: struct.Image.html
    fn node(
        &self,
        style: Style,
        image: &I,
        width: Option<u16>,
        height: Option<u16>,
        source: Option<Rectangle<u16>>,
    ) -> Node;

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
        image: &I,
        bounds: Rectangle<f32>,
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
