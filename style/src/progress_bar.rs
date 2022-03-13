//! Provide progress feedback to your users.
use iced_core::Background;
use crate::IcedTheme;

/// The appearance of a progress bar.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub bar: Background,
    pub border_radius: f32,
}

/// A set of rules that dictate the style of a progress bar.
pub trait StyleSheet<Theme> {
    fn style(&self, theme: &Theme) -> Style;
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn style(&self, theme: &Theme) -> Style {
        Style {
            background: theme.accent.into(),
            bar: theme.active.into(),
            border_radius: 5.0,
        }
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
