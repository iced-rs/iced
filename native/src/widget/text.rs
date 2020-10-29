//! Write some text for your users to read.
use crate::{
    layout, Color, Element, Hasher, HorizontalAlignment, Layout, Length, Point,
    Rectangle, Size, VerticalAlignment, Widget,
};

use std::hash::Hash;

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # type Text = iced_native::Text<iced_native::renderer::Null>;
/// #
/// Text::new("I <3 iced!")
///     .color([0.0, 0.0, 1.0])
///     .size(40);
/// ```
///
/// ![Text drawn by `iced_wgpu`](https://github.com/hecrj/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text.png?raw=true)
#[derive(Debug)]
pub struct Text<Renderer: self::Renderer> {
    content: String,
    size: Option<u16>,
    color: Option<Color>,
    font: Renderer::Font,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl<Renderer: self::Renderer> Text<Renderer> {
    /// Create a new fragment of [`Text`] with the given contents.
    ///
    /// [`Text`]: struct.Text.html
    pub fn new<T: Into<String>>(label: T) -> Self {
        Text {
            content: label.into(),
            size: None,
            color: None,
            font: Default::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// Sets the size of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`Color`]: ../../struct.Color.html
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    /// [`Font`]: ../../struct.Font.html
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = font.into();
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

impl<Message, Renderer> Widget<Message, Renderer> for Text<Renderer>
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
        let limits = limits.width(self.width).height(self.height);

        let size = self.size.unwrap_or(renderer.default_size());

        let bounds = limits.max();

        let (width, height) =
            renderer.measure(&self.content, size, self.font, bounds);

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            layout.bounds(),
            &self.content,
            self.size.unwrap_or(renderer.default_size()),
            self.font,
            self.color,
            self.horizontal_alignment,
            self.vertical_alignment,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

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
    /// The font type used for [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    type Font: Default + Copy;

    /// Returns the default size of [`Text`].
    ///
    /// [`Text`]: struct.Text.html
    fn default_size(&self) -> u16;

    /// Measures the [`Text`] in the given bounds and returns the minimum
    /// boundaries that can fit the contents.
    ///
    /// [`Text`]: struct.Text.html
    fn measure(
        &self,
        content: &str,
        size: u16,
        font: Self::Font,
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
        defaults: &Self::Defaults,
        bounds: Rectangle,
        content: &str,
        size: u16,
        font: Self::Font,
        color: Option<Color>,
        horizontal_alignment: HorizontalAlignment,
        vertical_alignment: VerticalAlignment,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<Text<Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: self::Renderer + 'a,
{
    fn from(text: Text<Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

impl<Renderer: self::Renderer> Clone for Text<Renderer> {
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            size: self.size,
            color: self.color,
            font: self.font,
            width: self.width,
            height: self.height,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
        }
    }
}
