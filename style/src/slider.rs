//! Display an interactive selector of a single value from a range of values.
use crate::Theme;
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
    pub border_width: f32,
    pub border_color: Color,
}

/// The shape of the handle of a slider.
#[derive(Debug, Clone, Copy)]
pub enum HandleShape {
    Circle { radius: f32 },
    Rectangle { width: u16, border_radius: f32 },
}

/// A set of rules that dictate the style of a slider.
pub trait StyleSheet {
    type Theme;
    fn get_style(
        &self,
        theme: &Self::Theme,
        is_dragging: bool,
        is_mouse_over: bool,
    ) -> Style {
        if is_dragging {
            self.dragging(theme)
        } else if is_mouse_over {
            self.hovered(theme)
        } else {
            self.active(theme)
        }
    }

    /// Produces the style of an active slider.
    fn active(&self, theme: &Self::Theme) -> Style;

    /// Produces the style of an hovered slider.
    fn hovered(&self, theme: &Self::Theme) -> Style;

    /// Produces the style of a slider that is being dragged.
    fn dragging(&self, theme: &Self::Theme) -> Style;
}

struct Default;

impl StyleSheet<IcedTheme> for Default {
    fn active(&self, theme: &Self::Theme) -> Style {
        Style {
            rail_colors: (
                Color {
                    a: 0.5,
                    ..theme.accent
                }
                .into(),
                Color::WHITE,
            ),
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 8,
                    border_radius: 4.0,
                },
                color: theme.surface,
                border_color: theme.accent,
                border_width: 1.0,
            },
        }
    }

    fn hovered(&self, theme: &Self::Theme) -> Style {
        let active = self.active(theme);

        Style {
            handle: Handle {
                color: theme.hover,
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self, theme: &Self::Theme) -> Style {
        let active = self.active(theme);

        Style {
            handle: Handle {
                color: Color::from_rgb(
                    theme.hover.r - 0.05,
                    theme.hover.g - 0.05,
                    theme.hover.b - 0.05,
                ),
                ..active.handle
            },
            ..active
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
