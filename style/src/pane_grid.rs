//! Let your users split regions of your application and organize layout
//! dynamically.
use iced_core::Color;

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    fn get_style(&self, is_picked: bool) -> Option<Style> {
        if is_picked {
            self.picked_split()
        } else {
            self.hovered_split()
        }
    }

    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self) -> Option<Style>;

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self) -> Option<Style>;
}

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

struct Default;

impl StyleSheet for Default {
    fn picked_split(&self) -> Option<Style> {
        None
    }

    fn hovered_split(&self) -> Option<Style> {
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
