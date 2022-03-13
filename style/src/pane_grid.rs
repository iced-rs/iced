//! Let your users split regions of your application and organize layout
//! dynamically.
use crate::IcedTheme;
use iced_core::Color;

/// A line.
///
/// It is normally used to define the highlight of something, like a split.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The [`Color`] of the line drawn.
    pub color: Color,

    /// The width of the line drawn.
    pub width: f32,
}

/// A set of rules that dictate the style of a pane grid.
pub trait StyleSheet {
    type Theme;
    fn get_style(&self, theme: &Self::Theme, is_picked: bool) -> Option<Style> {
        if is_picked {
            self.picked_split(theme)
        } else {
            self.hovered_split(theme)
        }
    }

    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self, theme: &Self::Theme) -> Option<Style>;

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self, theme: &Self::Theme) -> Option<Style>;
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn picked_split(&self, _theme: &Self::Theme) -> Option<Style> {
        None
    }

    fn hovered_split(&self, _theme: &Self::Theme) -> Option<Style> {
        None
    }
}

impl<'a> std::default::Default for Box<dyn StyleSheet + 'a> {
    fn default() -> Self {
        Box::new(Default)
    }
}

impl<'a, T> From<T> for Box<dyn StyleSheet + 'a>
where
    T: StyleSheet + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
