//! Create choices using tab buttons.
use iced_core::{Background, Color};

/// The appearance of a tab button.
#[derive(Debug)]
pub struct Style {
    pub background: Option<Background>,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub text_color: Color,
    // The style of an optional "active" indicator line.
    pub indicator: Option<Indicator>,
}

/// The position of the "active" [`Indicator`] line in a [`Tab`] button.
///
/// [`Indicator`]: struct.Indicator.html
/// [`Tab`]: struct.Tab.html
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Position {
    /// Along the bottom side of the [`Tab`] button.
    ///
    /// [`Tab`]: struct.Tab.html
    Bottom,
    /// Along the top side of the [`Tab`] button.
    ///
    /// [`Tab`]: struct.Tab.html
    Top,
    /// Along the left side of the [`Tab`] button.
    ///
    /// [`Tab`]: struct.Tab.html
    Left,
    /// Along the right of the [`Tab`] button.
    ///
    /// [`Tab`]: struct.Tab.html
    Right,
}

impl std::default::Default for Position {
    fn default() -> Self {
        Position::Bottom
    }
}

/// The appearance of an "active" indicator line in a tab button.
#[derive(Debug)]
pub struct Indicator {
    pub color: Color,
    /// The thickness of the line.
    pub thickness: f32,
    pub border_radius: f32,
    /// The length of the line along the side of the tab.
    ///
    /// Set this to `None` to use the full length of the tab.
    pub length: Option<u16>,
    /// The position of the indicator inside the tab button.
    pub position: Position,
    /// The offset from the edge of the tab.
    pub offset: u16,
}

impl std::default::Default for Indicator {
    fn default() -> Self {
        Indicator {
            color: Color::from_rgb(0.36, 0.36, 0.36),
            thickness: 2.0,
            border_radius: 0.0,
            length: None,
            position: Position::Bottom,
            offset: 0,
        }
    }
}

/// A set of rules that dictate the style of a tab button.
pub trait StyleSheet {
    fn selected(&self) -> Style;

    fn unselected(&self) -> Style;

    fn selected_hovered(&self) -> Style;

    fn unselected_hovered(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn selected(&self) -> Style {
        Style {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
            indicator: Some(Indicator::default()),
        }
    }

    fn unselected(&self) -> Style {
        Style {
            text_color: Color::from_rgb(0.34, 0.34, 0.34),
            indicator: None,
            ..self.selected()
        }
    }

    fn selected_hovered(&self) -> Style {
        Style {
            background: Some(Background::Color(Color::from_rgba(
                0.9, 0.9, 0.9, 0.9,
            ))),
            ..self.selected()
        }
    }

    fn unselected_hovered(&self) -> Style {
        Style {
            background: Some(Background::Color(Color::from_rgba(
                0.9, 0.9, 0.9, 0.9,
            ))),
            text_color: Color::from_rgb(0.34, 0.34, 0.34),
            indicator: None,
            ..self.selected()
        }
    }
}

/// The default style for vertical tabs.
pub struct StyleDefaultVertical;

impl StyleSheet for StyleDefaultVertical {
    fn selected(&self) -> Style {
        Style {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: Color::BLACK,
            indicator: Some(Indicator {
                position: Position::Left,
                ..Indicator::default()
            }),
        }
    }

    fn unselected(&self) -> Style {
        Style {
            text_color: Color::from_rgb(0.34, 0.34, 0.34),
            indicator: None,
            ..self.selected()
        }
    }

    fn selected_hovered(&self) -> Style {
        Style {
            background: Some(Background::Color(Color::from_rgba(
                0.9, 0.9, 0.9, 0.9,
            ))),
            ..self.selected()
        }
    }

    fn unselected_hovered(&self) -> Style {
        Style {
            background: Some(Background::Color(Color::from_rgba(
                0.9, 0.9, 0.9, 0.9,
            ))),
            text_color: Color::from_rgb(0.34, 0.34, 0.34),
            indicator: None,
            ..self.selected()
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
