//! Let your users split regions of your application and organize layout
//! dynamically.
use iced_core::Color;

/// A set of rules that dictate the style of a container.
pub trait StyleSheet {
    /// The [`Line`] to draw when a split is picked.
    fn picked_split(&self) -> Option<Line>;

    /// The [`Line`] to draw when a split is hovered.
    fn hovered_split(&self) -> Option<Line>;
}

/// A line.
///
/// It is normally used to define the highlight of something, like a split.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// The [`Color`] of the [`Line`].
    pub color: Color,

    /// The width of the [`Line`].
    pub width: f32,
}

struct Default;

impl StyleSheet for Default {
    fn picked_split(&self) -> Option<Line> {
        None
    }

    fn hovered_split(&self) -> Option<Line> {
        None
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
