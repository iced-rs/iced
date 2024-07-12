//! Show toggle controls using togglers.
use crate::core::alignment;
use crate::core::event;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::text;
use crate::core::touch;
use crate::core::widget;
use crate::core::widget::tree::{self, Tree};
use crate::core::{
    Border, Clipboard, Color, Element, Event, Layout, Length, Pixels,
    Rectangle, Shell, Size, Theme, Widget,
};

/// A toggler widget.
///
/// # Example
///
/// ```no_run
/// # type Toggler<'a, Message> = iced_widget::Toggler<'a, Message>;
/// #
/// pub enum Message {
///     TogglerToggled(bool),
/// }
///
/// let is_toggled = true;
///
/// Toggler::new(String::from("Toggle me!"), is_toggled, |b| Message::TogglerToggled(b));
/// ```
#[allow(missing_debug_implementations)]
pub struct Toggler<
    'a,
    Message,
    Theme = crate::Theme,
    Renderer = crate::Renderer,
> where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    is_toggled: bool,
    on_toggle: Box<dyn Fn(bool) -> Message + 'a>,
    label: Option<String>,
    width: Length,
    size: f32,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_alignment: alignment::Horizontal,
    text_shaping: text::Shaping,
    spacing: f32,
    font: Option<Renderer::Font>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> Toggler<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// The default size of a [`Toggler`].
    pub const DEFAULT_SIZE: f32 = 16.0;

    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is checked or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    pub fn new<F>(
        label: impl Into<Option<String>>,
        is_toggled: bool,
        f: F,
    ) -> Self
    where
        F: 'a + Fn(bool) -> Message,
    {
        Toggler {
            is_toggled,
            on_toggle: Box::new(f),
            label: label.into(),
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_alignment: alignment::Horizontal::Left,
            text_shaping: text::Shaping::Basic,
            spacing: Self::DEFAULT_SIZE / 2.0,
            font: None,
            class: Theme::default(),
        }
    }

    /// Sets the size of the [`Toggler`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Toggler`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the text size o the [`Toggler`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the [`Toggler`].
    pub fn text_line_height(
        mut self,
        line_height: impl Into<text::LineHeight>,
    ) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the text of the [`Toggler`]
    pub fn text_alignment(mut self, alignment: alignment::Horizontal) -> Self {
        self.text_alignment = alignment;
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Toggler`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the spacing between the [`Toggler`] and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the [`Renderer::Font`] of the text of the [`Toggler`]
    ///
    /// [`Renderer::Font`]: crate::core::text::Renderer
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Toggler`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Toggler`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toggler<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<widget::text::State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(widget::text::State::<Renderer::Paragraph>::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);

        layout::next_to_each_other(
            &limits,
            self.spacing,
            |_| layout::Node::new(Size::new(2.0 * self.size, self.size)),
            |limits| {
                if let Some(label) = self.label.as_deref() {
                    let state = tree
                    .state
                    .downcast_mut::<widget::text::State<Renderer::Paragraph>>();

                    widget::text::layout(
                        state,
                        renderer,
                        limits,
                        self.width,
                        Length::Shrink,
                        label,
                        self.text_line_height,
                        self.text_size,
                        self.font,
                        self.text_alignment,
                        alignment::Vertical::Top,
                        self.text_shaping,
                    )
                } else {
                    layout::Node::new(Size::ZERO)
                }
            },
        )
    }

    fn on_event(
        &mut self,
        _state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let mouse_over = cursor.is_over(layout.bounds());

                if mouse_over {
                    shell.publish((self.on_toggle)(!self.is_toggled));

                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        /// Makes sure that the border radius of the toggler looks good at every size.
        const BORDER_RADIUS_RATIO: f32 = 32.0 / 13.0;

        /// The space ratio between the background Quad and the Toggler bounds, and
        /// between the background Quad and foreground Quad.
        const SPACE_RATIO: f32 = 0.05;

        let mut children = layout.children();
        let toggler_layout = children.next().unwrap();

        if self.label.is_some() {
            let label_layout = children.next().unwrap();

            crate::text::draw(
                renderer,
                style,
                label_layout,
                tree.state.downcast_ref(),
                crate::text::Style::default(),
                viewport,
            );
        }

        let bounds = toggler_layout.bounds();
        let is_mouse_over = cursor.is_over(layout.bounds());

        let status = if is_mouse_over {
            Status::Hovered {
                is_toggled: self.is_toggled,
            }
        } else {
            Status::Active {
                is_toggled: self.is_toggled,
            }
        };

        let style = theme.style(&self.class, status);

        let border_radius = bounds.height / BORDER_RADIUS_RATIO;
        let space = SPACE_RATIO * bounds.height;

        let toggler_background_bounds = Rectangle {
            x: bounds.x + space,
            y: bounds.y + space,
            width: bounds.width - (2.0 * space),
            height: bounds.height - (2.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_background_bounds,
                border: Border {
                    radius: border_radius.into(),
                    width: style.background_border_width,
                    color: style.background_border_color,
                },
                ..renderer::Quad::default()
            },
            style.background,
        );

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x
                + if self.is_toggled {
                    bounds.width - 2.0 * space - (bounds.height - (4.0 * space))
                } else {
                    2.0 * space
                },
            y: bounds.y + (2.0 * space),
            width: bounds.height - (4.0 * space),
            height: bounds.height - (4.0 * space),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_foreground_bounds,
                border: Border {
                    radius: border_radius.into(),
                    width: style.foreground_border_width,
                    color: style.foreground_border_color,
                },
                ..renderer::Quad::default()
            },
            style.foreground,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Toggler<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        toggler: Toggler<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(toggler)
    }
}

/// The possible status of a [`Toggler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Toggler`] can be interacted with.
    Active {
        /// Indicates whether the [`Toggler`] is toggled.
        is_toggled: bool,
    },
    /// The [`Toggler`] is being hovered.
    Hovered {
        /// Indicates whether the [`Toggler`] is toggled.
        is_toggled: bool,
    },
}

/// The appearance of a toggler.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The background [`Color`] of the toggler.
    pub background: Color,
    /// The width of the background border of the toggler.
    pub background_border_width: f32,
    /// The [`Color`] of the background border of the toggler.
    pub background_border_color: Color,
    /// The foreground [`Color`] of the toggler.
    pub foreground: Color,
    /// The width of the foreground border of the toggler.
    pub foreground_border_width: f32,
    /// The [`Color`] of the foreground border of the toggler.
    pub foreground_border_color: Color,
}

/// The theme catalog of a [`Toggler`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Toggler`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Toggler`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let background = match status {
        Status::Active { is_toggled } | Status::Hovered { is_toggled } => {
            if is_toggled {
                palette.primary.strong.color
            } else {
                palette.background.strong.color
            }
        }
    };

    let foreground = match status {
        Status::Active { is_toggled } => {
            if is_toggled {
                palette.primary.strong.text
            } else {
                palette.background.base.color
            }
        }
        Status::Hovered { is_toggled } => {
            if is_toggled {
                Color {
                    a: 0.5,
                    ..palette.primary.strong.text
                }
            } else {
                palette.background.weak.color
            }
        }
    };

    Style {
        background,
        foreground,
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
    }
}
