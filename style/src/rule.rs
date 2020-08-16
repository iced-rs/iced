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

/// The appearance of a rule.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The color of the rule.
    pub color: Color,
    /// The width (thickness) of the rule line.
    pub width: u16,
    /// The radius of the rectangle corners.
    pub radius: u16,
    /// The [`FillMode`] of the rule.
    ///
    /// [`FillMode`]: enum.FillMode.html
    pub fill_mode: FillMode,
}

/// A set of rules that dictate the style of a rule.
pub trait StyleSheet {
    /// Produces the style of a rule.
    fn style(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn style(&self) -> Style {
        Style {
            color: [0.6, 0.6, 0.6, 0.51].into(),
            width: 1,
            radius: 0,
            fill_mode: FillMode::Percent(90.0),
        }
    }
}

impl std::default::Default for Box<dyn StyleSheet> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<T> From<T> for Box<dyn StyleSheet>
where
    T: 'static + StyleSheet,
{
    fn from(style: T) -> Self {
        Box::new(style)
    }
}
