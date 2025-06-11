//! Rules divide space horizontally or vertically.
//!
//! # Example
//! ```no_run
//! # mod iced { pub mod widget { pub use iced_widget::*; } }
//! # pub type State = ();
//! # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
//! use iced::widget::horizontal_rule;
//!
//! #[derive(Clone)]
//! enum Message {
//!     // ...,
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     horizontal_rule(2).into()
//! }
//! ```
use crate::core;
use crate::core::border;
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    Color, Element, Layout, Length, Pixels, Rectangle, Size, Theme, Widget,
};

/// Display a horizontal or vertical rule for dividing content.
///
/// # Example
/// ```no_run
/// # mod iced { pub mod widget { pub use iced_widget::*; } }
/// # pub type State = ();
/// # pub type Element<'a, Message> = iced_widget::core::Element<'a, Message, iced_widget::Theme, iced_widget::Renderer>;
/// use iced::widget::horizontal_rule;
///
/// #[derive(Clone)]
/// enum Message {
///     // ...,
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     horizontal_rule(2).into()
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Rule<'a, Theme = crate::Theme>
where
    Theme: Catalog,
{
    width: Length,
    height: Length,
    is_horizontal: bool,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Rule<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a horizontal [`Rule`] with the given height.
    pub fn horizontal(height: impl Into<Pixels>) -> Self {
        Rule {
            width: Length::Fill,
            height: Length::Fixed(height.into().0),
            is_horizontal: true,
            class: Theme::default(),
        }
    }

    /// Creates a vertical [`Rule`] with the given width.
    pub fn vertical(width: impl Into<Pixels>) -> Self {
        Rule {
            width: Length::Fixed(width.into().0),
            height: Length::Fill,
            is_horizontal: false,
            class: Theme::default(),
        }
    }

    /// Sets the style of the [`Rule`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Rule`].
    #[cfg(feature = "advanced")]
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Rule<'_, Theme>
where
    Renderer: core::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = theme.style(&self.class);

        let bounds = if self.is_horizontal {
            let line_y = (bounds.y + (bounds.height / 2.0)
                - (style.width as f32 / 2.0))
                .round();

            let (offset, line_width) = style.fill_mode.fill(bounds.width);
            let line_x = bounds.x + offset;

            Rectangle {
                x: line_x,
                y: line_y,
                width: line_width,
                height: style.width as f32,
            }
        } else {
            let line_x = (bounds.x + (bounds.width / 2.0)
                - (style.width as f32 / 2.0))
                .round();

            let (offset, line_height) = style.fill_mode.fill(bounds.height);
            let line_y = bounds.y + offset;

            Rectangle {
                x: line_x,
                y: line_y,
                width: style.width as f32,
                height: line_height,
            }
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: border::rounded(style.radius),
                snap: style.snap,
                ..renderer::Quad::default()
            },
            style.color,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Rule<'a, Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + core::Renderer,
{
    fn from(rule: Rule<'a, Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(rule)
    }
}

/// The appearance of a rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The color of the rule.
    pub color: Color,
    /// The width (thickness) of the rule line.
    pub width: u16,
    /// The radius of the line corners.
    pub radius: border::Radius,
    /// The [`FillMode`] of the rule.
    pub fill_mode: FillMode,
    /// Whether the rule should be snapped to the pixel grid.
    pub snap: bool,
}

/// The fill mode of a rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FillMode {
    /// Fill the whole length of the container.
    Full,
    /// Fill a percent of the length of the container. The rule
    /// will be centered in that container.
    ///
    /// The range is `[0.0, 100.0]`.
    Percent(f32),
    /// Uniform offset from each end, length units.
    Padded(u16),
    /// Different offset on each end of the rule, length units.
    /// First = top or left.
    AsymmetricPadding(u16, u16),
}

impl FillMode {
    /// Return the starting offset and length of the rule.
    ///
    /// * `space` - The space to fill.
    ///
    /// # Returns
    ///
    /// * (`starting_offset`, `length`)
    pub fn fill(&self, space: f32) -> (f32, f32) {
        match *self {
            FillMode::Full => (0.0, space),
            FillMode::Percent(percent) => {
                if percent >= 100.0 {
                    (0.0, space)
                } else {
                    let percent_width = (space * percent / 100.0).round();

                    (((space - percent_width) / 2.0).round(), percent_width)
                }
            }
            FillMode::Padded(padding) => {
                if padding == 0 {
                    (0.0, space)
                } else {
                    let padding = padding as f32;
                    let mut line_width = space - (padding * 2.0);
                    if line_width < 0.0 {
                        line_width = 0.0;
                    }

                    (padding, line_width)
                }
            }
            FillMode::AsymmetricPadding(first_pad, second_pad) => {
                let first_pad = first_pad as f32;
                let second_pad = second_pad as f32;
                let mut line_width = space - first_pad - second_pad;
                if line_width < 0.0 {
                    line_width = 0.0;
                }

                (first_pad, line_width)
            }
        }
    }
}

/// The theme catalog of a [`Rule`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Rule`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

/// The default styling of a [`Rule`].
pub fn default(theme: &Theme) -> Style {
    let palette = theme.extended_palette();

    Style {
        color: palette.background.strong.color,
        width: 1,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
        snap: true,
    }
}
