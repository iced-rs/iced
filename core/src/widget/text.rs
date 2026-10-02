//! Text widgets display information through writing.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub fn text<T>(t: T) -> iced_core::widget::Text<'static, iced_core::Theme> { unimplemented!() } }
//! #            pub use iced_core::color; }
//! # pub trait Widget<Message>: iced_core::Widget<Message, iced_core::Theme, ()> {}
//! # impl<T, Message> Widget<Message> for T where T: iced_core::Widget<Message, iced_core::Theme, ()> {}
//! # pub type State = ();
//! use iced::widget::text;
//! use iced::color;
//!
//! enum Message {
//!     // ...
//! }
//!
//! fn view(state: &State) -> impl Widget<Message> {
//!     text("Hello, this is iced!")
//!         .size(20)
//!         .color(color!(0x0000ff))
//! }
//! ```
use crate::alignment;
use crate::layout;
use crate::mouse;
use crate::renderer;
use crate::text::paragraph::{self, Paragraph};
use crate::text::{self, Target};
use crate::widget;
use crate::widget::operation;
use crate::widget::tree::{self, Tree};
use crate::window;
use crate::{Color, Event, Font, Layout, Length, Pixels, Point, Rectangle, Size, Theme, Widget};

pub use text::{Alignment, Ellipsis, LineHeight, Position, Shaping, Wrapping};

/// A bunch of text.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub fn text<T>(t: T) -> iced_core::widget::Text<'static, iced_core::Theme> { unimplemented!() } }
/// #            pub use iced_core::color; }
/// # pub trait Widget<Message>: iced_core::Widget<Message, iced_core::Theme, ()> {}
/// # impl<T, Message> Widget<Message> for T where T: iced_core::Widget<Message, iced_core::Theme, ()> {}
/// # pub type State = ();
/// use iced::widget::text;
/// use iced::color;
///
/// enum Message {
///     // ...
/// }
///
/// fn view(state: &State) -> impl Widget<Message> {
///     text("Hello, this is iced!")
///         .size(20)
///         .color(color!(0x0000ff))
/// }
/// ```
#[must_use]
pub struct Text<'a, Theme>
where
    Theme: Catalog,
{
    fragment: text::Fragment<'a>,
    format: Format,
    selectable: bool,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Text<'a, Theme>
where
    Theme: Catalog,
{
    /// Create a new fragment of [`Text`] with the given contents.
    pub fn new(fragment: impl text::IntoFragment<'a>) -> Self {
        Text {
            fragment: fragment.into_fragment(),
            format: Format::default(),
            selectable: false,
            class: Theme::default(),
        }
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.format.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`Text`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.format.line_height = Some(line_height.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    pub fn font(mut self, font: impl Into<Font>) -> Self {
        self.format.font = Some(font.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`], if `Some`.
    pub fn font_maybe(mut self, font: Option<impl Into<Font>>) -> Self {
        self.format.font = font.map(Into::into);
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.format.width = width.into();
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.format.height = height.into();
        self
    }

    /// Centers the [`Text`], both horizontally and vertically.
    pub fn center(self) -> Self {
        self.align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
    }

    /// Sets the [`alignment::Horizontal`] of the [`Text`].
    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.format.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Text`].
    pub fn align_y(mut self, alignment: impl Into<alignment::Vertical>) -> Self {
        self.format.align_y = alignment.into();
        self
    }

    /// Sets the [`Shaping`] strategy of the [`Text`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.format.shaping = shaping;
        self
    }

    /// Sets the [`Wrapping`] strategy of the [`Text`].
    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.format.wrapping = wrapping;
        self
    }

    /// Sets the [`Ellipsis`] strategy of the [`Text`].
    pub fn ellipsis(mut self, ellipsis: Ellipsis) -> Self {
        self.format.ellipsis = ellipsis;
        self
    }

    /// Sets whether the [`Text`] can be selected.
    ///
    /// By default, it is `false`.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Sets the style of the [`Text`].
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    /// Sets the [`Color`] of the [`Text`], if `Some`.
    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| Style {
            color,
            selection: None,
        })
    }

    /// Sets the style class of the [`Text`].
    #[cfg(feature = "advanced")]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Theme> widget::Meta for Text<'_, Theme> where Theme: Catalog {}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Text<'_, Theme>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.format.width,
            height: self.format.height,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::default())
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        tree.size = layout(
            &mut state.paragraph,
            renderer,
            limits,
            &self.fragment,
            self.format,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &crate::Event,
        layout: Layout,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _shell: &mut crate::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        update::<Renderer::Paragraph>(tree, event, layout, cursor);
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        _layout: Layout,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if self.selectable {
            let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

            if state.is_hovered {
                return mouse::Interaction::Text;
            }
        }

        mouse::Interaction::None
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout,
        _cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = theme.style(&self.class);

        draw(
            renderer,
            defaults,
            layout.bounds(),
            state.paragraph.raw(),
            style,
            theme.selection(),
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        _viewport: &Rectangle,
        _renderer: &Renderer,
        operation: &mut dyn super::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        operation.text(
            None,
            layout.bounds(),
            &mut Operand {
                paragraph: &mut state.paragraph,
                layout,
                selectable: self.selectable,
            },
        );
    }
}

impl widget::Meta for &str {}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for &str
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fit,
            height: Length::Fit,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::default())
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &layout::Limits) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        tree.size = layout(
            &mut state.paragraph,
            renderer,
            limits,
            self,
            Format::default(),
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout,
        _cursor_position: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let style = theme.style(&Theme::default());

        draw(
            renderer,
            defaults,
            layout.bounds(),
            state.paragraph.raw(),
            style,
            theme.selection(),
            viewport,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout,
        _viewport: &Rectangle,
        _renderer: &Renderer,
        operation: &mut dyn super::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
        operation.text(
            None,
            layout.bounds(),
            &mut Operand {
                paragraph: &mut state.paragraph,
                layout,
                selectable: false,
            },
        );
    }
}

#[derive(Default)]
struct State<P: Paragraph> {
    paragraph: paragraph::Plain<P>,
    is_hovered: bool,
}

/// The state of a widget with text, operated on by [`super::Operation::text`].
pub struct Operand<'a, P: Paragraph> {
    /// The [`Paragraph`] of the widget.
    pub paragraph: &'a mut paragraph::Plain<P>,

    /// The layout of the widget.
    pub layout: Layout,

    /// Whether the text of the widget can be selected.
    pub selectable: bool,
}

impl<P: Paragraph> operation::Text for Operand<'_, P> {
    fn text(&self) -> text::Fragment<'_> {
        self.paragraph.content().into()
    }

    fn select(&mut self, start: Point, end: Point, target: Target) {
        if !self.selectable {
            return;
        }

        let anchor = self.layout.bounds().anchor(
            self.paragraph.min_bounds(),
            self.paragraph.align_x(),
            self.paragraph.align_y(),
        );

        let translation = anchor - Point::ORIGIN;

        self.paragraph
            .raw_mut()
            .select(start - translation, end - translation, target);
    }

    fn select_all(&mut self) {
        if !self.selectable {
            return;
        }

        self.paragraph.raw_mut().select_all();
    }

    fn deselect(&mut self) {
        self.paragraph.raw_mut().deselect();
    }

    fn copy(&mut self) -> Option<String> {
        self.paragraph.raw_mut().copy()
    }
}

/// The format of some [`Text`].
///
/// Check out the methods of the [`Text`] widget
/// to learn more about each field.
#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct Format {
    pub width: Length,
    pub height: Length,
    pub size: Option<Pixels>,
    pub font: Option<Font>,
    pub line_height: Option<LineHeight>,
    pub align_x: text::Alignment,
    pub align_y: alignment::Vertical,
    pub shaping: Shaping,
    pub wrapping: Wrapping,
    pub ellipsis: Ellipsis,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            size: None,
            line_height: None,
            font: None,
            width: Length::Fit,
            height: Length::Fit,
            align_x: text::Alignment::Default,
            align_y: alignment::Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
            ellipsis: Ellipsis::default(),
        }
    }
}

fn update<P: Paragraph + 'static>(
    tree: &mut Tree,
    event: &crate::Event,
    layout: Layout,
    cursor: mouse::Cursor,
) {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { .. })
        | Event::Window(window::Event::RedrawRequested(_)) => {
            let state = tree.state.downcast_mut::<State<P>>();

            let Some(position) = cursor.position_in(layout.bounds()) else {
                state.is_hovered = false;
                return;
            };

            let anchor = Rectangle::with_size(layout.size()).anchor(
                state.paragraph.min_bounds(),
                state.paragraph.align_x(),
                state.paragraph.align_y(),
            );

            let translation = anchor - Point::ORIGIN;

            state.is_hovered = state.paragraph.raw().hit_glyph(position - translation);
        }
        _ => {}
    }
}

/// Computes the [`Size`] of a [`Text`] widget.
pub fn layout<Renderer>(
    paragraph: &mut paragraph::Plain<Renderer::Paragraph>,
    renderer: &Renderer,
    limits: &layout::Limits,
    content: &str,
    format: Format,
) -> Size
where
    Renderer: text::Renderer,
{
    layout::sized(limits, format.width, format.height, |limits| {
        let bounds = limits.bounds();

        let size = format.size.unwrap_or_else(|| renderer.text_size());
        let font = format.font.unwrap_or_else(|| renderer.font());
        let line_height = format.line_height.unwrap_or_else(|| renderer.line_height());

        let _ = paragraph.update(text::Text {
            content,
            bounds,
            size,
            line_height,
            font,
            align_x: format.align_x,
            align_y: format.align_y,
            shaping: format.shaping,
            wrapping: format.wrapping,
            ellipsis: format.ellipsis,
            hint_factor: renderer.hint_factor(),
        });

        paragraph.min_bounds()
    })
}

/// Draws text using the same logic as the [`Text`] widget.
pub fn draw<Renderer>(
    renderer: &mut Renderer,
    style: &renderer::Style,
    bounds: Rectangle,
    paragraph: &Renderer::Paragraph,
    appearance: Style,
    selection_color: Color,
    viewport: &Rectangle,
) where
    Renderer: text::Renderer,
{
    let anchor = bounds.anchor(
        paragraph.min_bounds(),
        paragraph.align_x(),
        paragraph.align_y(),
    );

    renderer.fill_paragraph(
        paragraph,
        anchor,
        appearance.color.unwrap_or(style.text_color),
        *viewport,
    );

    let selection = paragraph.selection();

    if !selection.is_empty() {
        let translation = anchor - Point::ORIGIN;

        for region in selection {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: *region + translation,
                    ..renderer::Quad::default()
                },
                appearance.selection.unwrap_or(selection_color),
            );
        }
    }
}

impl<'a, Theme> From<&'a str> for Text<'a, Theme>
where
    Theme: Catalog + 'a,
{
    fn from(content: &'a str) -> Self {
        Self::new(content)
    }
}

/// The appearance of some text.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    /// The [`Color`] of the text.
    ///
    /// The default, `None`, means using the inherited color.
    pub color: Option<Color>,

    /// The [`Color`] of the selection, if any.
    ///
    /// The default, `None`, means using the global selection color.
    pub selection: Option<Color>,
}

/// The theme catalog of a [`Text`].
pub trait Catalog: Sized {
    /// The item class of this [`Catalog`].
    type Class<'a>;

    /// The default class produced by this [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, item: &Self::Class<'_>) -> Style;

    /// The global selection [`Color`].
    fn selection(&self) -> Color;
}

/// A styling function for a [`Text`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_theme| Style::default())
    }

    fn selection(&self) -> Color {
        self.palette().background.strongest.color
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default text styling; color is inherited.
pub fn default(_theme: &Theme) -> Style {
    Style {
        color: None,
        selection: None,
    }
}

/// Text with the default base color.
pub fn base(theme: &Theme) -> Style {
    Style {
        color: Some(theme.seed().text),
        selection: None,
    }
}

/// Text conveying some important information, like an action.
pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.seed().primary),
        selection: None,
    }
}

/// Text conveying some secondary information, like a footnote.
pub fn secondary(theme: &Theme) -> Style {
    Style {
        color: Some(theme.palette().secondary.base.color),
        selection: None,
    }
}

/// Text conveying some positive information, like a successful event.
pub fn success(theme: &Theme) -> Style {
    Style {
        color: Some(theme.seed().success),
        selection: None,
    }
}

/// Text conveying some mildly negative information, like a warning.
pub fn warning(theme: &Theme) -> Style {
    Style {
        color: Some(theme.seed().warning),
        selection: None,
    }
}

/// Text conveying some negative information, like an error.
pub fn danger(theme: &Theme) -> Style {
    Style {
        color: Some(theme.seed().danger),
        selection: None,
    }
}
