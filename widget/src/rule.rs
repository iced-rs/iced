//! Display a horizontal or vertical rule for dividing content.
use crate::core::border::{self, Border};
use crate::core::layout;
use crate::core::mouse;
use crate::core::renderer;
use crate::core::widget::Tree;
use crate::core::{
    Color, Element, Layout, Length, Pixels, Rectangle, Size, Theme, Widget,
};

/// Display a horizontal or vertical rule for dividing content.
#[allow(missing_debug_implementations)]
pub struct Rule<Theme = crate::Theme> {
    width: Length,
    height: Length,
    is_horizontal: bool,
    style: Style<Theme>,
}

impl<Theme> Rule<Theme> {
    /// Creates a horizontal [`Rule`] with the given height.
    pub fn horizontal(height: impl Into<Pixels>) -> Self
    where
        Style<Theme>: Default,
    {
        Rule {
            width: Length::Fill,
            height: Length::Fixed(height.into().0),
            is_horizontal: true,
            style: Style::default(),
        }
    }

    /// Creates a vertical [`Rule`] with the given width.
    pub fn vertical(width: impl Into<Pixels>) -> Self
    where
        Style<Theme>: Default,
    {
        Rule {
            width: Length::Fixed(width.into().0),
            height: Length::Fill,
            is_horizontal: false,
            style: Style::default(),
        }
    }

    /// Sets the style of the [`Rule`].
    pub fn style(mut self, style: fn(&Theme) -> Appearance) -> Self {
        self.style = Style(style);
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Rule<Theme>
where
    Renderer: crate::core::Renderer,
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
        let appearance = (self.style.0)(theme);

        let bounds = if self.is_horizontal {
            let line_y = (bounds.y + (bounds.height / 2.0)
                - (appearance.width as f32 / 2.0))
                .round();

            let (offset, line_width) = appearance.fill_mode.fill(bounds.width);
            let line_x = bounds.x + offset;

            Rectangle {
                x: line_x,
                y: line_y,
                width: line_width,
                height: appearance.width as f32,
            }
        } else {
            let line_x = (bounds.x + (bounds.width / 2.0)
                - (appearance.width as f32 / 2.0))
                .round();

            let (offset, line_height) =
                appearance.fill_mode.fill(bounds.height);
            let line_y = bounds.y + offset;

            Rectangle {
                x: line_x,
                y: line_y,
                width: appearance.width as f32,
                height: line_height,
            }
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border::with_radius(appearance.radius),
                ..renderer::Quad::default()
            },
            appearance.color,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Rule<Theme>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: 'a + crate::core::Renderer,
{
    fn from(rule: Rule<Theme>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(rule)
    }
}

/// The appearance of a rule.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The color of the rule.
    pub color: Color,
    /// The width (thickness) of the rule line.
    pub width: u16,
    /// The radius of the line corners.
    pub radius: border::Radius,
    /// The [`FillMode`] of the rule.
    pub fill_mode: FillMode,
}

/// The fill mode of a rule.
#[derive(Debug, Clone, Copy)]
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

/// The style of a [`Rule`].
#[derive(Debug, PartialEq, Eq)]
pub struct Style<Theme>(fn(&Theme) -> Appearance);

impl<Theme> Clone for Style<Theme> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Theme> Copy for Style<Theme> {}

impl Default for Style<Theme> {
    fn default() -> Self {
        Style(default)
    }
}

impl<Theme> From<fn(&Theme) -> Appearance> for Style<Theme> {
    fn from(f: fn(&Theme) -> Appearance) -> Self {
        Style(f)
    }
}

/// The default styling of a [`Rule`].
pub fn default(theme: &Theme) -> Appearance {
    let palette = theme.extended_palette();

    Appearance {
        color: palette.background.strong.color,
        width: 1,
        radius: 0.0.into(),
        fill_mode: FillMode::Full,
    }
}
