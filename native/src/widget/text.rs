//! Write some text for your users to read.
use crate::{
    layout, Element, Font, Hasher, HorizontalAlignment, Layout, Length, Point,
    Rectangle, Size, VerticalAlignment, Widget,
};

use std::hash::Hash;

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # use iced_native::Text;
/// #
/// Text::new("I <3 iced!")
///     .color([0.0, 0.0, 1.0])
///     .size(40);
/// ```
///
/// ![Text drawn by `iced_wgpu`](https://github.com/hecrj/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text.png?raw=true)
#[derive(Debug, Clone)]
pub struct Text<Style> {
    content: String,
    font: Font,
    size: u16,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    style: Style,
}

impl<Style> Text<Style> {
    /// Create a new fragment of [`Text`] with the given contents.
    ///
    /// [`Text`]: struct.Text.html
    pub fn new(label: impl Into<String>) -> Self
    where
        Style: Default,
    {
        Text {
            content: label.into(),
            style: Style::default(),
            font: Font::Default,
            size: 20,
            width: Length::Fill,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// Create a new fragment of [`Text`] with the given contents and a custom
    /// `style`.
    ///
    /// [`Text`]: struct.Text.html
    /// [`Palette`]: ../struct.Palette.html
    pub fn new_with_style(label: impl Into<String>, style: Style) -> Self {
        Text {
            content: label.into(),
            style,
            font: Font::Default,
            size: 20,
            width: Length::Fill,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// Changes the style of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    pub fn change_style(mut self, f: impl FnOnce(&mut Style)) -> Self {
        f(&mut self.style);
        self
    }

    /// Sets the size of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`Font`]: ../../struct.Font.html
    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    ///
    /// [`Text`]: struct.Text.html
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    ///
    /// [`Text`]: struct.Text.html
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
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

impl<Message, Renderer, Style> Widget<Message, Renderer> for Text<Style>
where
    Renderer: self::Renderer<WidgetStyle = Style>,
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
        let limits = limits.width(self.width).height(self.height);

        let bounds = limits.max();

        let (width, height) =
            renderer.measure(&self.content, self.size, self.font, bounds);

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        layout: Layout<'_>,
        _cursor_position: Point,
    ) -> Renderer::Output {
        renderer.draw(
            layout.bounds(),
            &self.content,
            self.size,
            self.font,
            &self.style,
            self.horizontal_alignment,
            self.vertical_alignment,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        self.content.hash(state);
        self.size.hash(state);
        self.width.hash(state);
        self.height.hash(state);
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
pub trait Renderer: crate::Renderer {
    /// Struct that consists of all style options the renderer supports for
    /// [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    type WidgetStyle;

    /// Measures the [`Text`] in the given bounds and returns the minimum
    /// boundaries that can fit the contents.
    ///
    /// [`Text`]: struct.Text.html
    fn measure(
        &self,
        content: &str,
        size: u16,
        font: Font,
        bounds: Size,
    ) -> (f32, f32);

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
        size: u16,
        font: Font,
        style: &Self::WidgetStyle,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self::Output;
}

impl<'a, Message, Renderer, Style> From<Text<Style>> for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer<WidgetStyle = Style>,
    Style: 'static,
{
    fn from(text: Text<Style>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}
