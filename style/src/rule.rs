//! Display a horizontal or vertical rule for dividing content.
use iced_core::Color;

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
    /// * (starting_offset, length)
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

/// The appearance of a rule.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The color of the rule.
    pub color: Color,
    /// The width (thickness) of the rule line.
    pub width: u16,
    /// The radius of the line corners.
    pub radius: f32,
    /// The [`FillMode`] of the rule.
    pub fill_mode: FillMode,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Style {
            color: [0.6, 0.6, 0.6, 0.6].into(),
            width: 1,
            radius: 0.0,
            fill_mode: FillMode::Full,
        }
    }
}

/// A set of rules that dictate the style of a rule.
pub trait StyleSheet {
    /// Produces the style of a rule.
    fn style(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn style(&self) -> Style {
        Style::default()
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: 'a + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
