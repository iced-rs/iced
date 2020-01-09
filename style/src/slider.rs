//! Display an interactive selector of a single value from a range of values.
use iced_core::Color;

/// The appearance of a slider.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub rail_colors: (Color, Color),
    pub handle: Handle,
}

/// The appearance of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub struct Handle {
    pub shape: HandleShape,
    pub color: Color,
    pub border_width: u16,
    pub border_color: Color,
}

/// The shape of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub enum HandleShape {
    Circle { radius: u16 },
    Rectangle { width: u16, border_radius: u16 },
}

/// A set of rules that dictate the style of a slider.
pub trait StyleSheet {
    /// Produces the style of an active slider.
    fn active(&self) -> Style;

    /// Produces the style of an hovered slider.
    fn hovered(&self) -> Style;

    /// Produces the style of a slider that is being dragged.
    fn dragging(&self) -> Style;
}

struct Default;

impl StyleSheet for Default {
    fn active(&self) -> Style {
        Style {
            rail_colors: ([0.6, 0.6, 0.6, 0.5].into(), Color::WHITE),
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 8,
                    border_radius: 4,
                },
                color: Color::from_rgb(0.95, 0.95, 0.95),
                border_color: Color::from_rgb(0.6, 0.6, 0.6),
                border_width: 1,
            },
        }
    }

    fn hovered(&self) -> Style {
        let active = self.active();

        Style {
            handle: Handle {
                color: Color::from_rgb(0.90, 0.90, 0.90),
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self) -> Style {
        let active = self.active();

        Style {
            handle: Handle {
                color: Color::from_rgb(0.85, 0.85, 0.85),
                ..active.handle
            },
            ..active
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
