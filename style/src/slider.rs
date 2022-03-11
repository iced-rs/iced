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
pub trait StyleSheet<ColorPalette> {
    fn get_style(
        &self,
        color_palette: &ColorPalette,
        is_dragging: bool,
        is_mouse_over: bool,
    ) -> Style {
        if is_dragging {
            self.dragging(color_palette)
        } else if is_mouse_over {
            self.hovered(color_palette)
        } else {
            self.active(color_palette)
        }
    }

    /// Produces the style of an active slider.
    fn active(&self, color_palette: &ColorPalette) -> Style;

    /// Produces the style of an hovered slider.
    fn hovered(&self, color_palette: &ColorPalette) -> Style;

    /// Produces the style of a slider that is being dragged.
    fn dragging(&self, color_palette: &ColorPalette) -> Style;
}

struct Default;

impl StyleSheet<IcedColorPalette> for Default {
    fn active(&self, color_palette: &ColorPalette) -> Style {
        Style {
            rail_colors: (
                Color {
                    a: 0.5,
                    ..color_palette.accent
                }
                .into(),
                Color::WHITE,
            ),
            handle: Handle {
                shape: HandleShape::Rectangle {
                    width: 8,
                    border_radius: 4.0,
                },
                color: color_palette.surface,
                border_color: color_palette.accent,
                border_width: 1.0,
            },
        }
    }

    fn hovered(&self, color_palette: &ColorPalette) -> Style {
        let active = self.active(color_palette);

        Style {
            handle: Handle {
                color: color_palette.hover,
                ..active.handle
            },
            ..active
        }
    }

    fn dragging(&self, color_palette: &ColorPalette) -> Style {
        let active = self.active(color_palette);

        Style {
            handle: Handle {
                color: Color::from_rgb(
                    color_palette.hover.r - 0.05,
                    color_palette.hover.g - 0.05,
                    color_palette.hover.b - 0.05,
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
