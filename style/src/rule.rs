//! Display a horizontal or vertical rule for dividing content.

use iced_core::Color;

/// The appearance of a rule.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The color of the rule.
    pub color: Color,
    /// The width (thickness) of the rule line.
    pub width: u16,
    /// The radius of the rectangle corners.
    pub radius: u16,
    /// The percent from [0, 100] of the filled space the rule
    /// will be drawn.
    pub fill_percent: u16,
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
            color: [0.6, 0.6, 0.6, 0.49].into(),
            width: 1,
            radius: 0,
            fill_percent: 90,
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
