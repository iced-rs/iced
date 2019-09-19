//! Write some text for your users to read.
use crate::{
    Color, Element, Hasher, Layout, MouseCursor, Node, Point, Rectangle, Style,
    Widget,
};

use std::hash::Hash;

/// A paragraph of text.
///
/// It implements [`Widget`] when the associated `Renderer` implements the
/// [`text::Renderer`] trait.
///
/// [`Widget`]: ../trait.Widget.html
/// [`text::Renderer`]: trait.Renderer.html
///
/// # Example
///
/// ```
/// use iced::{Text, Color};
///
/// Text::new("I <3 iced!")
///     .size(40);
/// ```
#[derive(Debug, Clone)]
pub struct Text {
    /// The text contents
    pub content: String,
    /// The text size
    pub size: Option<u16>,
    color: Option<Color>,
    style: Style,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Text {
    /// Create a new fragment of [`Text`] with the given contents.
    ///
    /// [`Text`]: struct.Text.html
    pub fn new(label: &str) -> Self {
        Text {
            content: String::from(label),
            size: None,
            color: None,
            style: Style::default().fill_width(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// Sets the size of the [`Text`] in pixels.
    ///
    /// [`Text`]: struct.Text.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the `Color` of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the width of the [`Text`] boundaries in pixels.
    ///
    /// [`Text`]: struct.Text.html
    pub fn width(mut self, width: u16) -> Self {
        self.style = self.style.width(width);
        self
    }

    /// Sets the height of the [`Text`] boundaries in pixels.
    ///
    /// [`Text`]: struct.Text.html
    pub fn height(mut self, height: u16) -> Self {
        self.style = self.style.height(height);
        self
    }

    /// Sets the [`HorizontalAlignment`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`HorizontalAlignment`]: enum.HorizontalAlignment.html
    pub fn horizontal_alignment(
        mut self,
        alignment: HorizontalAlignment,
    ) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`VerticalAlignment`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`VerticalAlignment`]: enum.VerticalAlignment.html
    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<Message, Renderer> Widget<Message, Renderer> for Text
where
    Renderer: self::Renderer,
{
    fn node(&self, renderer: &mut Renderer) -> Node {
        renderer.node(self.style, &self.content, self.size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> MouseCursor {
        renderer.draw(
            layout.bounds(),
            &self.content,
            self.size,
            self.color,
            self.horizontal_alignment,
            self.vertical_alignment,
        );

        MouseCursor::OutOfBounds
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.style.hash(state);

        self.content.hash(state);
        self.size.hash(state);
    }
}

/// The renderer of a [`Text`] fragment.
///
/// Your [renderer] will need to implement this trait before being
/// able to use [`Text`] in your [`UserInterface`].
///
/// [`Text`]: struct.Text.html
/// [renderer]: ../../renderer/index.html
/// [`UserInterface`]: ../../struct.UserInterface.html
pub trait Renderer {
    /// Creates a [`Node`] with the given [`Style`] for the provided [`Text`]
    /// contents and size.
    ///
    /// You should probably use [`Node::with_measure`] to allow [`Text`] to
    /// adapt to the dimensions of its container.
    ///
    /// [`Node`]: ../../struct.Node.html
    /// [`Style`]: ../../struct.Style.html
    /// [`Text`]: struct.Text.html
    /// [`Node::with_measure`]: ../../struct.Node.html#method.with_measure
    fn node(&self, style: Style, content: &str, size: Option<u16>) -> Node;

    /// Draws a [`Text`] fragment.
    ///
    /// It receives:
    ///   * the bounds of the [`Text`]
    ///   * the contents of the [`Text`]
    ///   * the size of the [`Text`]
    ///   * the color of the [`Text`]
    ///   * the [`HorizontalAlignment`] of the [`Text`]
    ///   * the [`VerticalAlignment`] of the [`Text`]
    ///
    /// [`Text`]: struct.Text.html
    /// [`HorizontalAlignment`]: enum.HorizontalAlignment.html
    /// [`VerticalAlignment`]: enum.VerticalAlignment.html
    fn draw(
        &mut self,
        bounds: Rectangle,
        content: &str,
        size: Option<u16>,
        color: Option<Color>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    );
}

impl<'a, Message, Renderer> From<Text> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    fn from(text: Text) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

/// The horizontal alignment of some resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    /// Align left
    Left,

    /// Horizontally centered
    Center,

    /// Align right
    Right,
}

/// The vertical alignment of some resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    /// Align top
    Top,

    /// Vertically centered
    Center,

    /// Align bottom
    Bottom,
}
