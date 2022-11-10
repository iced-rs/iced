//! Write some text for your users to read.
use crate::alignment;
use crate::layout;
use crate::renderer;
use crate::text;
use crate::widget::Tree;
use crate::{Element, Layout, Length, Point, Rectangle, Size, Widget};

use std::borrow::Cow;

pub use iced_style::text::{Appearance, StyleSheet};

/// A paragraph of text.
///
/// # Example
///
/// ```
/// # use iced_native::Color;
/// #
/// # type Text<'a> = iced_native::widget::Text<'a, iced_native::renderer::Null>;
/// #
/// Text::new("I <3 iced!")
///     .size(40)
///     .style(Color::from([0.0, 0.0, 1.0]));
/// ```
///
/// ![Text drawn by `iced_wgpu`](https://github.com/iced-rs/iced/blob/7760618fb112074bc40b148944521f312152012a/docs/images/text.png?raw=true)
#[allow(missing_debug_implementations)]
pub struct Text<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    content: Cow<'a, str>,
    size: Option<u16>,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Renderer::Font,
    style: <Renderer::Theme as StyleSheet>::Style,
}

impl<'a, Renderer> Text<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self {
        Text {
            content: content.into(),
            size: None,
            font: Default::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            style: Default::default(),
        }
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = font.into();
        self
    }

    /// Sets the style of the [`Text`].
    pub fn style(
        mut self,
        style: impl Into<<Renderer::Theme as StyleSheet>::Style>,
    ) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`alignment::Horizontal`] of the [`Text`].
    pub fn horizontal_alignment(
        mut self,
        alignment: alignment::Horizontal,
    ) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Text`].
    pub fn vertical_alignment(
        mut self,
        alignment: alignment::Vertical,
    ) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for Text<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
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

        let size = self.size.unwrap_or_else(|| renderer.default_size());

        let bounds = limits.max();

        let (width, height) =
            renderer.measure(&self.content, size, self.font.clone(), bounds);

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) {
        draw(
            renderer,
            style,
            layout,
            &self.content,
            self.size,
            self.font.clone(),
            theme.appearance(self.style),
            self.horizontal_alignment,
            self.vertical_alignment,
        );
    }
}

/// Draws text using the same logic as the [`Text`] widget.
///
/// Specifically:
///
/// * If no `size` is provided, the default text size of the `Renderer` will be
///   used.
/// * If no `color` is provided, the [`renderer::Style::text_color`] will be
///   used.
/// * The alignment attributes do not affect the position of the bounds of the
///   [`Layout`].
pub fn draw<Renderer>(
    renderer: &mut Renderer,
    style: &renderer::Style,
    layout: Layout<'_>,
    content: &str,
    size: Option<u16>,
    font: Renderer::Font,
    appearance: Appearance,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
) where
    Renderer: text::Renderer,
{
    let bounds = layout.bounds();

    let x = match horizontal_alignment {
        alignment::Horizontal::Left => bounds.x,
        alignment::Horizontal::Center => bounds.center_x(),
        alignment::Horizontal::Right => bounds.x + bounds.width,
    };

    let y = match vertical_alignment {
        alignment::Vertical::Top => bounds.y,
        alignment::Vertical::Center => bounds.center_y(),
        alignment::Vertical::Bottom => bounds.y + bounds.height,
    };

    renderer.fill_text(crate::text::Text {
        content,
        size: f32::from(size.unwrap_or_else(|| renderer.default_size())),
        bounds: Rectangle { x, y, ..bounds },
        color: appearance.color.unwrap_or(style.text_color),
        font,
        horizontal_alignment,
        vertical_alignment,
    });
}

impl<'a, Message, Renderer> From<Text<'a, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(text: Text<'a, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(text)
    }
}

impl<'a, Renderer> Clone for Text<'a, Renderer>
where
    Renderer: text::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn clone(&self) -> Self {
        Self {
            content: self.content.clone(),
            size: self.size,
            width: self.width,
            height: self.height,
            horizontal_alignment: self.horizontal_alignment,
            vertical_alignment: self.vertical_alignment,
            font: self.font.clone(),
            style: self.style,
        }
    }
}

impl<'a, Message, Renderer> From<&'a str> for Element<'a, Message, Renderer>
where
    Renderer: text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
{
    fn from(contents: &'a str) -> Self {
        Text::new(contents).into()
    }
}
