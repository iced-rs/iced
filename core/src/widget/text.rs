//! Write some text for your users to read.
use crate::alignment;
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text::{self, Paragraph};
use crate::widget::tree::{self, Tree};
use crate::{
    Color, Element, Layout, Length, Pixels, Point, Rectangle, Size, Theme,
    Widget,
};

use std::borrow::Cow;

pub use text::{LineHeight, Shaping};

/// A paragraph of text.
#[allow(missing_debug_implementations)]
pub struct Text<'a, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    content: Cow<'a, str>,
    size: Option<Pixels>,
    line_height: LineHeight,
    width: Length,
    height: Length,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    font: Option<Renderer::Font>,
    shaping: Shaping,
    style: Style<'a, Theme>,
}

impl<'a, Theme, Renderer> Text<'a, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new(content: impl Into<Cow<'a, str>>) -> Self
    where
        Theme: DefaultStyle + 'a,
    {
        Text {
            content: content.into(),
            size: None,
            line_height: LineHeight::default(),
            font: None,
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: alignment::Horizontal::Left,
            vertical_alignment: alignment::Vertical::Top,
            shaping: Shaping::Basic,
            style: Box::new(Theme::default_style),
        }
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`Text`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Text`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Appearance + 'a) -> Self {
        self.style = Box::new(style);
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color(self, color: impl Into<Color>) -> Self {
        self.color_maybe(Some(color))
    }

    /// Sets the [`Color`] of the [`Text`], if `Some`.
    pub fn color_maybe(mut self, color: Option<impl Into<Color>>) -> Self {
        let color = color.map(Into::into);

        self.style = Box::new(move |_theme| Appearance { color });
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
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

    /// Sets the [`Shaping`] strategy of the [`Text`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.shaping = shaping;
        self
    }
}

/// The internal state of a [`Text`] widget.
#[derive(Debug, Default)]
pub struct State<P: Paragraph>(P);

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Text<'a, Theme, Renderer>
where
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State(Renderer::Paragraph::default()))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            tree.state.downcast_mut::<State<Renderer::Paragraph>>(),
            renderer,
            limits,
            self.width,
            self.height,
            &self.content,
            self.line_height,
            self.size,
            self.font,
            self.horizontal_alignment,
            self.vertical_alignment,
            self.shaping,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let appearance = (self.style)(theme);

        draw(renderer, style, layout, state, appearance, viewport);
    }
}

/// Produces the [`layout::Node`] of a [`Text`] widget.
pub fn layout<Renderer>(
    state: &mut State<Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    width: Length,
    height: Length,
    content: &str,
    line_height: LineHeight,
    size: Option<Pixels>,
    font: Option<Renderer::Font>,
    horizontal_alignment: alignment::Horizontal,
    vertical_alignment: alignment::Vertical,
    shaping: Shaping,
) -> layout::Node
where
    Renderer: text::Renderer,
{
    layout::sized(limits, width, height, |limits| {
        let bounds = limits.max();

        let size = size.unwrap_or_else(|| renderer.default_size());
        let font = font.unwrap_or_else(|| renderer.default_font());

        let State(ref mut paragraph) = state;

        paragraph.update(text::Text {
            content,
            bounds,
            size,
            line_height,
            font,
            horizontal_alignment,
            vertical_alignment,
            shaping,
        });

        paragraph.min_bounds()
    })
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
    state: &State<Renderer::Paragraph>,
    appearance: Appearance,
    viewport: &Rectangle,
) where
    Renderer: text::Renderer,
{
    let State(ref paragraph) = state;
    let bounds = layout.bounds();

    let x = match paragraph.horizontal_alignment() {
        alignment::Horizontal::Left => bounds.x,
        alignment::Horizontal::Center => bounds.center_x(),
        alignment::Horizontal::Right => bounds.x + bounds.width,
    };

    let y = match paragraph.vertical_alignment() {
        alignment::Vertical::Top => bounds.y,
        alignment::Vertical::Center => bounds.center_y(),
        alignment::Vertical::Bottom => bounds.y + bounds.height,
    };

    renderer.fill_paragraph(
        paragraph,
        Point::new(x, y),
        appearance.color.unwrap_or(style.text_color),
        *viewport,
    );
}

impl<'a, Message, Theme, Renderer> From<Text<'a, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        text: Text<'a, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(text)
    }
}

impl<'a, Theme, Renderer> From<&'a str> for Text<'a, Theme, Renderer>
where
    Theme: DefaultStyle + 'a,
    Renderer: text::Renderer,
{
    fn from(content: &'a str) -> Self {
        Self::new(content)
    }
}

impl<'a, Message, Theme, Renderer> From<&'a str>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: DefaultStyle + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(content: &'a str) -> Self {
        Text::from(content).into()
    }
}

/// The appearance of some text.
#[derive(Debug, Clone, Copy, Default)]
pub struct Appearance {
    /// The [`Color`] of the text.
    ///
    /// The default, `None`, means using the inherited color.
    pub color: Option<Color>,
}

/// The style of some [`Text`].
pub type Style<'a, Theme> = Box<dyn Fn(&Theme) -> Appearance + 'a>;

/// The default style of some [`Text`].
pub trait DefaultStyle {
    /// Returns the default style of some [`Text`].
    fn default_style(&self) -> Appearance;
}

impl DefaultStyle for Theme {
    fn default_style(&self) -> Appearance {
        Appearance::default()
    }
}

impl DefaultStyle for Color {
    fn default_style(&self) -> Appearance {
        Appearance { color: Some(*self) }
    }
}
