//! Navigate an endless amount of content with a scrollbar.
use iced_core::{Background, Color};

/// The appearance of a scrollable.
#[derive(Debug, Clone)]
pub struct Scrollbar {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub scroller: Scroller,
}

/// The appearance of the scroller of a scrollable.
#[derive(Debug, Clone, Copy)]
pub struct Scroller {
    pub color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
}

/// A set of rules that dictate the style of a scrollable.
pub trait StyleSheet {
    /// Produces the style of an active scrollbar.
    fn active(&self) -> Scrollbar;

    /// Produces the style of an hovered scrollbar.
    fn hovered(&self) -> Scrollbar;

    /// Produces the style of a scrollbar that is being dragged.
    fn dragging(&self) -> Scrollbar {
        self.hovered()
    }
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Scrollbar {
        Scrollbar {
            background: None,
            border_radius: 5.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: Scroller {
                color: [0.0, 0.0, 0.0, 0.7].into(),
                border_radius: 5.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(&self) -> Scrollbar {
        Scrollbar {
            background: Some(Background::Color([0.0, 0.0, 0.0, 0.3].into())),
            ..self.active()
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
    T: StyleSheet + 'a,
{
    fn from(style_sheet: T) -> Self {
        Box::new(style_sheet)
    }
}
